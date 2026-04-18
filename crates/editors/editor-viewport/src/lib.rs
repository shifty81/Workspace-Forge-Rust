//! wgpu-backed 3-D viewport for the NovaForge Scene Editor.
//!
//! Renders a perspective-projected grid on the XZ plane together with
//! colour-coded X/Y/Z axis lines, and wireframe box markers for every scene
//! entity, all inside an egui rect using [`egui_wgpu::Callback`].
//! The calling panel supplies a [`CameraState`] that controls the orbit camera
//! and a slice of [`EntityMarker`] values that position the entity boxes.
//!
//! # Integration
//!
//! 1. Call [`init_viewport_pipeline`] **once** from inside the
//!    [`eframe::CreationContext`] closure, passing
//!    `cc.wgpu_render_state.as_ref().unwrap()`.
//! 2. Each frame, call [`paint_viewport`] from your panel's `ui()` method,
//!    passing the painter, the already-allocated viewport rect, the current
//!    [`CameraState`], and the list of [`EntityMarker`] values to render.

use bytemuck::{Pod, Zeroable};
use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};
use wgpu::util::DeviceExt;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of entity wireframe boxes that can be rendered per frame.
/// Each box uses 24 vertices (12 edges × 2 end-points as `LineList`).
const MAX_ENTITY_BOXES: usize = 512;
const VERTS_PER_BOX: usize = 24; // 12 edges × 2 vertices
const MAX_ENTITY_VERTS: usize = MAX_ENTITY_BOXES * VERTS_PER_BOX;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Describes a single entity that should be rendered as a wireframe box in the
/// viewport.  Build a `Vec<EntityMarker>` from your scene entities and pass it
/// to [`paint_viewport`] every frame.
#[derive(Clone, Copy)]
pub struct EntityMarker {
    /// World-space position of the entity's origin.
    pub position: [f32; 3],
    /// Whether this entity is currently selected.  Selected entities are drawn
    /// with a brighter highlight colour.
    pub selected: bool,
}

/// Orbit-camera state.  Store one instance in the panel that owns the
/// viewport and pass it to [`paint_viewport`] every frame.
#[derive(Clone, Copy)]
pub struct CameraState {
    /// Horizontal orbit angle around the Y axis, in radians.
    pub yaw: f32,
    /// Vertical elevation angle, in radians.  Clamped to approximately
    /// ±1.40 radians (±80 °) before use to prevent gimbal lock.
    pub pitch: f32,
    /// Distance from `center` to the camera eye, in world units.
    pub distance: f32,
    /// World-space point the camera looks at.
    pub center: [f32; 3],
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            yaw: 0.6,
            pitch: 0.5,
            distance: 20.0,
            center: [0.0; 3],
        }
    }
}

impl CameraState {
    /// Compute the combined view-projection matrix (column-major).
    pub fn view_proj(&self, aspect: f32) -> [[f32; 4]; 4] {
        let eye = self.eye_position();
        let view = look_at(eye, self.center, [0.0, 1.0, 0.0]);
        let proj = perspective(FRAC_PI_3, aspect, 0.1, 500.0);
        mat4_mul(proj, view)
    }

    fn eye_position(&self) -> [f32; 3] {
        let pitch = self.pitch.clamp(-FRAC_PI_2 * 0.95, FRAC_PI_2 * 0.95);
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let d = self.distance;
        [
            self.center[0] + d * sy * cp,
            self.center[1] + d * sp,
            self.center[2] + d * cy * cp,
        ]
    }
}

/// Initialise the wgpu render pipeline and store the resources inside the
/// egui renderer's [`CallbackResources`] map.
///
/// Must be called exactly once from inside the `CreationContext` closure:
///
/// ```rust,ignore
/// editor_viewport::init_viewport_pipeline(cc.wgpu_render_state.as_ref().unwrap());
/// ```
pub fn init_viewport_pipeline(render_state: &RenderState) {
    let device = &render_state.device;

    // ── Shader ───────────────────────────────────────────────────────────────
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("viewport_shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
    });

    // ── Uniform buffer (view-projection matrix, 64 bytes) ────────────────────
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("viewport_uniforms"),
        size: std::mem::size_of::<[f32; 16]>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("viewport_bgl"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("viewport_bg"),
        layout: &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    // ── Vertex buffer (grid + axes, static geometry) ─────────────────────────
    let vertices = build_grid_vertices();
    let vertex_count = vertices.len() as u32;
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("viewport_vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // ── Entity box vertex buffer (dynamic, re-written each frame) ─────────────
    let entity_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("viewport_entity_vertices"),
        size: (MAX_ENTITY_VERTS * std::mem::size_of::<Vertex>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // ── Render pipeline ───────────────────────────────────────────────────────
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("viewport_layout"),
        bind_group_layouts: &[&bgl],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("viewport_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: render_state.target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    // ── Store in renderer callback resources ─────────────────────────────────
    render_state
        .renderer
        .write()
        .callback_resources
        .insert(ViewportResources {
            pipeline,
            vertex_buffer,
            vertex_count,
            uniform_buffer,
            bind_group,
            entity_vertex_buffer,
            entity_vertex_count: 0,
        });
}

/// Add a wgpu paint callback to `painter` that renders the 3-D grid viewport
/// covering `rect` with the supplied `camera` and entity markers.
///
/// You must have called [`init_viewport_pipeline`] at startup, and `rect`
/// must already have been allocated by the caller (`ui.allocate_exact_size`
/// or similar).  If the wgpu resources are absent (e.g. running without a
/// wgpu backend) this is a no-op.
///
/// Each [`EntityMarker`] in `entities` is drawn as a 0.8-unit wireframe box
/// at its world-space position.  Selected entities use a brighter colour.
pub fn paint_viewport(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: CameraState,
    entities: &[EntityMarker],
) {
    painter.add(egui_wgpu::Callback::new_paint_callback(
        rect,
        ViewportCallback {
            camera,
            viewport_rect: rect,
            entities: entities.to_vec(),
        },
    ));
}

// ---------------------------------------------------------------------------
// Vertex layout
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

fn vert(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> Vertex {
    Vertex {
        position: [x, y, z],
        color: [r, g, b, 1.0],
    }
}

// ---------------------------------------------------------------------------
// Grid + axis geometry
// ---------------------------------------------------------------------------

fn build_grid_vertices() -> Vec<Vertex> {
    let mut v = Vec::new();

    let half = 10i32;
    let dim = [0.22_f32, 0.22, 0.28]; // minor grid colour
    let mid = [0.38_f32, 0.38, 0.48]; // major (axis-aligned at zero)

    // Lines parallel to the Z axis (varying X)
    for xi in -half..=half {
        let x = xi as f32;
        let [r, g, b] = if xi == 0 { mid } else { dim };
        v.push(vert(x, 0.0, -(half as f32), r, g, b));
        v.push(vert(x, 0.0, half as f32, r, g, b));
    }
    // Lines parallel to the X axis (varying Z)
    for zi in -half..=half {
        let z = zi as f32;
        let [r, g, b] = if zi == 0 { mid } else { dim };
        v.push(vert(-(half as f32), 0.0, z, r, g, b));
        v.push(vert(half as f32, 0.0, z, r, g, b));
    }

    // Axis lines — brighter and slightly longer than the grid
    let al = (half + 2) as f32;
    // X — red
    v.push(vert(-al, 0.0, 0.0, 0.80, 0.20, 0.20));
    v.push(vert(al, 0.0, 0.0, 0.80, 0.20, 0.20));
    // Y — green
    v.push(vert(0.0, -al, 0.0, 0.20, 0.75, 0.20));
    v.push(vert(0.0, al, 0.0, 0.20, 0.75, 0.20));
    // Z — blue
    v.push(vert(0.0, 0.0, -al, 0.25, 0.45, 0.90));
    v.push(vert(0.0, 0.0, al, 0.25, 0.45, 0.90));

    v
}

// ---------------------------------------------------------------------------
// GPU resources (stored in egui_wgpu CallbackResources)
// ---------------------------------------------------------------------------

struct ViewportResources {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Pre-allocated buffer for entity wireframe boxes (MAX_ENTITY_VERTS × Vertex).
    entity_vertex_buffer: wgpu::Buffer,
    /// Number of entity vertices to draw this frame (written by `prepare()`).
    entity_vertex_count: u32,
}

// ---------------------------------------------------------------------------
// Paint callback
// ---------------------------------------------------------------------------

struct ViewportCallback {
    camera: CameraState,
    viewport_rect: egui::Rect,
    /// Entity markers to render as wireframe boxes this frame.
    entities: Vec<EntityMarker>,
}

impl CallbackTrait for ViewportCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(res) = callback_resources.get_mut::<ViewportResources>() else {
            return vec![];
        };

        let w = self.viewport_rect.width();
        let h = self.viewport_rect.height();
        let aspect = if h > 0.0 { w / h } else { 1.0 };

        let vp_flat = mat4_to_flat(self.camera.view_proj(aspect));
        queue.write_buffer(&res.uniform_buffer, 0, bytemuck::cast_slice(&vp_flat));

        // Build entity wireframe box vertices and upload them.
        let capped = self.entities.len().min(MAX_ENTITY_BOXES);
        let mut entity_verts: Vec<Vertex> = Vec::with_capacity(capped * VERTS_PER_BOX);
        for marker in &self.entities[..capped] {
            build_box_wireframe(marker, &mut entity_verts);
        }
        let count = entity_verts.len() as u32;
        res.entity_vertex_count = count;
        if count > 0 {
            queue.write_buffer(
                &res.entity_vertex_buffer,
                0,
                bytemuck::cast_slice(&entity_verts),
            );
        }

        vec![]
    }

    fn paint(
        &self,
        info: PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(res) = callback_resources.get::<ViewportResources>() else {
            return;
        };

        // Map NDC to the viewport panel's pixel region so the 3-D projection
        // matches the aspect ratio we used when building the view-proj matrix.
        let vp = info.viewport_in_pixels();
        render_pass.set_viewport(
            vp.left_px as f32,
            vp.top_px as f32,
            vp.width_px as f32,
            vp.height_px as f32,
            0.0,
            1.0,
        );

        render_pass.set_pipeline(&res.pipeline);
        render_pass.set_bind_group(0, &res.bind_group, &[]);

        // Draw the static grid + axis lines.
        render_pass.set_vertex_buffer(0, res.vertex_buffer.slice(..));
        render_pass.draw(0..res.vertex_count, 0..1);

        // Draw entity wireframe boxes (dynamic, rebuilt each frame).
        if res.entity_vertex_count > 0 {
            render_pass.set_vertex_buffer(0, res.entity_vertex_buffer.slice(..));
            render_pass.draw(0..res.entity_vertex_count, 0..1);
        }
    }
}

// ---------------------------------------------------------------------------
// WGSL shader source
// ---------------------------------------------------------------------------

const SHADER_SRC: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertIn {
    @location(0) position: vec3<f32>,
    @location(1) color:    vec4<f32>,
}

struct VertOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       color:    vec4<f32>,
}

@vertex
fn vs_main(in: VertIn) -> VertOut {
    var out: VertOut;
    out.clip_pos = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.color    = in.color;
    return out;
}

@fragment
fn fs_main(in: VertOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

// ---------------------------------------------------------------------------
// Entity box wireframe helper
// ---------------------------------------------------------------------------

/// Half-size of each wireframe box in world units (box is 0.8 × 0.8 × 0.8).
const BOX_HALF: f32 = 0.4;

/// Normal entity box colour (dim teal).
const BOX_NORMAL: [f32; 4] = [0.20, 0.75, 0.80, 1.0];
/// Selected entity box colour (bright orange-yellow).
const BOX_SELECTED: [f32; 4] = [1.00, 0.70, 0.10, 1.0];

/// Append 24 `LineList` vertices (12 wireframe edges) for a box centred at
/// `marker.position` to `out`.  Uses [`BOX_NORMAL`] or [`BOX_SELECTED`]
/// depending on `marker.selected`.
fn build_box_wireframe(marker: &EntityMarker, out: &mut Vec<Vertex>) {
    let [cx, cy, cz] = marker.position;
    let h = BOX_HALF;
    let col = if marker.selected {
        BOX_SELECTED
    } else {
        BOX_NORMAL
    };
    let [r, g, b, a] = col;

    // 8 corners of the box.
    let c = [
        [cx - h, cy - h, cz - h], // 0: ---
        [cx + h, cy - h, cz - h], // 1: +--
        [cx + h, cy - h, cz + h], // 2: +-+
        [cx - h, cy - h, cz + h], // 3: --+
        [cx - h, cy + h, cz - h], // 4: -+-
        [cx + h, cy + h, cz - h], // 5: ++-
        [cx + h, cy + h, cz + h], // 6: +++
        [cx - h, cy + h, cz + h], // 7: -++
    ];

    // 12 edges (each a pair of corner indices).
    let edges: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0), // bottom face
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4), // top face
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7), // vertical edges
    ];

    for (a_idx, b_idx) in edges {
        let [ax, ay, az] = c[a_idx];
        let [bx, by, bz] = c[b_idx];
        out.push(Vertex {
            position: [ax, ay, az],
            color: [r, g, b, a],
        });
        out.push(Vertex {
            position: [bx, by, bz],
            color: [r, g, b, a],
        });
    }
}

// ---------------------------------------------------------------------------
// Camera / matrix math (no external math crate required)
// ---------------------------------------------------------------------------

/// Perspective projection matrix (column-major, wgpu NDC depth [0, 1]).
fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y * 0.5).tan();
    let rng = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far * rng, -1.0],
        [0.0, 0.0, near * far * rng, 0.0],
    ]
}

/// Look-at view matrix (column-major).
fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize3(sub3(center, eye));
    let r = normalize3(cross3(f, up));
    let u = cross3(r, f);
    [
        [r[0], u[0], -f[0], 0.0],
        [r[1], u[1], -f[1], 0.0],
        [r[2], u[2], -f[2], 0.0],
        [-dot3(r, eye), -dot3(u, eye), dot3(f, eye), 1.0],
    ]
}

/// Column-major 4×4 matrix multiply: C = A * B.
fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut c = [[0.0f32; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            for k in 0..4 {
                c[col][row] += a[k][row] * b[col][k];
            }
        }
    }
    c
}

/// Flatten a column-major `[[f32;4];4]` to a `[f32;16]` for GPU upload.
fn mat4_to_flat(m: [[f32; 4]; 4]) -> [f32; 16] {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            out[col * 4 + row] = m[col][row];
        }
    }
    out
}

// 3-D vector helpers
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = dot3(v, v).sqrt();
    if len > 1e-8 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}
