//! Material / Shader Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// Hit-test radius for port circles, in screen pixels (zoom-independent).
const PORT_RADIUS: f32 = 9.0;

// ---------------------------------------------------------------------------
// Wire
// ---------------------------------------------------------------------------

/// A wire connecting an output port on one node to an input port on another.
#[derive(Clone)]
struct Wire {
    /// Source node index (output side).
    from_node: usize,
    /// Destination node index (input side).
    to_node: usize,
    /// Which input slot on `to_node` (0-based).
    to_input: usize,
}

// ---------------------------------------------------------------------------
// DragMode
// ---------------------------------------------------------------------------

/// What the canvas drag gesture is currently doing.
#[derive(Default, Clone, Copy, Debug)]
enum DragMode {
    #[default]
    Idle,
    /// Dragging a node to a new position.
    MovingNode(usize),
    /// Panning the canvas.
    PanningCanvas,
    /// Dragging a wire from an output port.
    DrawingWire {
        from_node: usize,
        /// Screen-space origin of the wire (output port centre).
        from_pos: egui::Pos2,
    },
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Screen-space rect for a node given the current pan/zoom.
fn node_screen_rect(
    canvas_rect: egui::Rect,
    pan: egui::Vec2,
    zoom: f32,
    node: &MaterialNode,
    node_w: f32,
    node_h: f32,
) -> egui::Rect {
    let top_left = canvas_rect.min + pan + node.pos.to_vec2() * zoom;
    egui::Rect::from_min_size(top_left, egui::vec2(node_w, node_h))
}

/// Screen-space centre of a node's output port (right edge).
fn output_port_pos(rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(rect.right(), rect.center().y)
}

/// Screen-space centre of a node's input port at `input_idx`.
fn input_port_pos(rect: egui::Rect, input_idx: usize, num_inputs: usize) -> egui::Pos2 {
    let y = rect.min.y + (input_idx as f32 + 1.0) * rect.height() / (num_inputs as f32 + 1.0);
    egui::pos2(rect.left(), y)
}

/// Draw a cubic Bézier curve as a polyline (24 segments).
fn draw_bezier(
    painter: &egui::Painter,
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    color: Color32,
    width: f32,
) {
    const N: usize = 24;
    let mut prev = p0;
    for i in 1..=N {
        let t = i as f32 / N as f32;
        let u = 1.0 - t;
        let next = egui::pos2(
            u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x + t * t * t * p3.x,
            u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y + t * t * t * p3.y,
        );
        painter.line_segment([prev, next], egui::Stroke::new(width, color));
        prev = next;
    }
}

// ---------------------------------------------------------------------------
// MaterialNode
// ---------------------------------------------------------------------------

/// A node in the material graph.
#[derive(Clone)]
struct MaterialNode {
    #[allow(dead_code)]
    id: usize,
    label: String,
    pos: egui::Pos2,
    inputs: Vec<String>,
    output: String,
}

// ---------------------------------------------------------------------------
// MaterialEditor
// ---------------------------------------------------------------------------

/// Material / Shader Editor panel.
///
/// Provides a node-graph canvas where nodes can be added, moved, and wired
/// together.  Drag from an output port (right side) to an input port (left
/// side) to create a wire.  Scroll to zoom; drag the canvas background to pan.
pub struct MaterialEditor {
    nodes: Vec<MaterialNode>,
    zoom: f32,
    pan: egui::Vec2,
    selected_node: Option<usize>,
    /// Next unique node ID.
    next_id: usize,
    /// Current canvas drag mode.
    drag_mode: DragMode,
    /// Wire connections between node ports.
    connections: Vec<Wire>,
}

impl Default for MaterialEditor {
    fn default() -> Self {
        Self {
            nodes: vec![
                MaterialNode {
                    id: 0,
                    label: "Texture Sample".to_string(),
                    pos: egui::pos2(80.0, 100.0),
                    inputs: vec!["UV".to_string()],
                    output: "RGBA".to_string(),
                },
                MaterialNode {
                    id: 1,
                    label: "Multiply".to_string(),
                    pos: egui::pos2(260.0, 120.0),
                    inputs: vec!["A".to_string(), "B".to_string()],
                    output: "Result".to_string(),
                },
                MaterialNode {
                    id: 2,
                    label: "Output".to_string(),
                    pos: egui::pos2(440.0, 140.0),
                    inputs: vec!["Colour".to_string(), "Alpha".to_string()],
                    output: String::new(),
                },
            ],
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            selected_node: None,
            next_id: 3,
            drag_mode: DragMode::Idle,
            // Seed with two default wires matching the original hard-coded
            // demonstration: Texture Sample → Multiply A → Output Colour.
            // Wire indices are validated with `.get()` during drawing, so
            // stale entries after node deletion are silently skipped.
            connections: vec![
                Wire { from_node: 0, to_node: 1, to_input: 0 },
                Wire { from_node: 1, to_node: 2, to_input: 0 },
            ],
        }
    }
}

impl EditorPanel for MaterialEditor {
    fn title(&self) -> &str {
        "Material Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
        // ── Toolbar ─────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.button("＋ Add Node").clicked() {
                let id = self.next_id;
                self.next_id += 1;
                let offset = egui::vec2(20.0, 20.0) * id as f32;
                self.nodes.push(MaterialNode {
                    id,
                    label: format!("Node {id}"),
                    pos: egui::pos2(80.0, 80.0) + offset,
                    inputs: vec!["In".to_string()],
                    output: "Out".to_string(),
                });
                self.selected_node = Some(self.nodes.len() - 1);
            }
            let delete_enabled = self.selected_node.is_some();
            if ui
                .add_enabled(delete_enabled, egui::Button::new("🗑 Delete Node"))
                .clicked()
            {
                if let Some(idx) = self.selected_node {
                    self.nodes.remove(idx);
                    // Prune wires touching the deleted node and fix indices.
                    self.connections.retain(|w| w.from_node != idx && w.to_node != idx);
                    for w in &mut self.connections {
                        if w.from_node > idx {
                            w.from_node -= 1;
                        }
                        if w.to_node > idx {
                            w.to_node -= 1;
                        }
                    }
                    self.selected_node = if self.nodes.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.nodes.len() - 1))
                    };
                }
            }
            ui.separator();
            if ui.button("🔍＋ Zoom In").clicked() {
                self.zoom = (self.zoom + 0.1).min(3.0);
            }
            if ui.button("🔍− Zoom Out").clicked() {
                self.zoom = (self.zoom - 0.1).max(0.3);
            }
            if ui.button("⊙ Reset View").clicked() {
                self.zoom = 1.0;
                self.pan = egui::Vec2::ZERO;
            }
            ui.separator();
            if ui
                .button("🗑 Clear Wires")
                .on_hover_text("Remove all wire connections")
                .clicked()
            {
                self.connections.clear();
            }
            if let Some(idx) = self.selected_node {
                if let Some(node) = self.nodes.get(idx) {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Selected: {}", node.label))
                            .size(11.0)
                            .color(Color32::from_rgb(140, 200, 255)),
                    );
                }
            }
            if matches!(self.drag_mode, DragMode::DrawingWire { .. }) {
                ui.separator();
                ui.label(
                    egui::RichText::new("🔌 Drawing wire — release on an input port to connect")
                        .size(11.0)
                        .color(Color32::from_rgb(220, 210, 80)),
                );
            }
        });

        ui.separator();

        // ── Canvas allocation ────────────────────────────────────────────────
        let available = ui.available_size();
        let (canvas_rect, response) =
            ui.allocate_exact_size(available, egui::Sense::click_and_drag());

        let node_w = 120.0 * self.zoom;
        let node_h = 64.0 * self.zoom;

        // Scroll-wheel zoom (canvas must be hovered).
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                self.zoom = (self.zoom * (1.0 + scroll * 0.003)).clamp(0.3, 3.0);
            }
        }

        // ── Drag handling ────────────────────────────────────────────────────
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut new_mode = DragMode::PanningCanvas;

                // Priority 1: output port → start wire drawing.
                'port: for (idx, node) in self.nodes.iter().enumerate() {
                    if node.output.is_empty() {
                        continue;
                    }
                    let rect =
                        node_screen_rect(canvas_rect, self.pan, self.zoom, node, node_w, node_h);
                    let port_pos = output_port_pos(rect);
                    if pos.distance(port_pos) < PORT_RADIUS {
                        new_mode = DragMode::DrawingWire {
                            from_node: idx,
                            from_pos: port_pos,
                        };
                        break 'port;
                    }
                }

                // Priority 2: node body → move node.
                if matches!(new_mode, DragMode::PanningCanvas) {
                    for (idx, node) in self.nodes.iter().enumerate() {
                        let rect = node_screen_rect(
                            canvas_rect, self.pan, self.zoom, node, node_w, node_h,
                        );
                        if rect.contains(pos) {
                            self.selected_node = Some(idx);
                            new_mode = DragMode::MovingNode(idx);
                            break;
                        }
                    }
                }

                self.drag_mode = new_mode;
            }
        }

        if response.dragged() {
            let delta = response.drag_delta();
            match self.drag_mode {
                DragMode::MovingNode(idx) => {
                    if let Some(node) = self.nodes.get_mut(idx) {
                        node.pos += delta / self.zoom;
                    }
                }
                DragMode::PanningCanvas => {
                    self.pan += delta;
                }
                DragMode::DrawingWire { .. } | DragMode::Idle => {}
            }
        }

        if response.drag_stopped() {
            // If a wire was being drawn, try to connect it to the nearest input.
            if let DragMode::DrawingWire { from_node, .. } = self.drag_mode {
                let drop_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(pos) = drop_pos {
                    'connect: for (to_idx, to_node) in self.nodes.iter().enumerate() {
                        if to_idx == from_node {
                            continue;
                        }
                        let rect = node_screen_rect(
                            canvas_rect, self.pan, self.zoom, to_node, node_w, node_h,
                        );
                        let num = to_node.inputs.len();
                        for input_idx in 0..num {
                            let port_pos = input_port_pos(rect, input_idx, num);
                            if pos.distance(port_pos) < PORT_RADIUS {
                                // Skip if this input is already wired.
                                let taken = self
                                    .connections
                                    .iter()
                                    .any(|w| w.to_node == to_idx && w.to_input == input_idx);
                                if !taken {
                                    self.connections.push(Wire {
                                        from_node,
                                        to_node: to_idx,
                                        to_input: input_idx,
                                    });
                                }
                                break 'connect;
                            }
                        }
                    }
                }
            }
            self.drag_mode = DragMode::Idle;
        }

        // ── Painting ─────────────────────────────────────────────────────────
        let painter = ui.painter_at(canvas_rect);
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(20, 20, 26));

        // Dot-grid background
        let step = 24.0 * self.zoom;
        let origin = canvas_rect.min + self.pan;
        let mut x = canvas_rect.left() + origin.x % step;
        while x < canvas_rect.right() {
            let mut y = canvas_rect.top() + origin.y % step;
            while y < canvas_rect.bottom() {
                painter.circle_filled(egui::pos2(x, y), 1.0, Color32::from_rgb(50, 50, 62));
                y += step;
            }
            x += step;
        }

        // ── Existing wire connections ─────────────────────────────────────────
        for wire in &self.connections {
            let from_rect = self
                .nodes
                .get(wire.from_node)
                .map(|n| node_screen_rect(canvas_rect, self.pan, self.zoom, n, node_w, node_h));
            let to_info = self.nodes.get(wire.to_node).map(|n| {
                let r = node_screen_rect(canvas_rect, self.pan, self.zoom, n, node_w, node_h);
                (r, n.inputs.len())
            });
            if let (Some(fr), Some((tr, num_in))) = (from_rect, to_info) {
                let p0 = output_port_pos(fr);
                let p3 = input_port_pos(tr, wire.to_input, num_in);
                let ctrl = ((p3.x - p0.x).abs() * 0.5).max(60.0);
                let p1 = egui::pos2(p0.x + ctrl, p0.y);
                let p2 = egui::pos2(p3.x - ctrl, p3.y);
                draw_bezier(&painter, p0, p1, p2, p3, Color32::from_rgb(180, 150, 60), 2.0);
            }
        }

        // ── In-progress wire ─────────────────────────────────────────────────
        if let DragMode::DrawingWire { from_pos, .. } = self.drag_mode {
            if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                let ctrl = ((cursor.x - from_pos.x).abs() * 0.5).max(60.0);
                let p1 = egui::pos2(from_pos.x + ctrl, from_pos.y);
                let p2 = egui::pos2(cursor.x - ctrl, cursor.y);
                draw_bezier(&painter, from_pos, p1, p2, cursor, Color32::from_rgb(230, 220, 80), 2.0);
                ui.ctx().request_repaint();
            }
        }

        // ── Nodes ─────────────────────────────────────────────────────────────
        let is_wiring = matches!(self.drag_mode, DragMode::DrawingWire { .. });
        let mut new_selected = self.selected_node;
        for (idx, node) in self.nodes.iter().enumerate() {
            let rect =
                node_screen_rect(canvas_rect, self.pan, self.zoom, node, node_w, node_h);
            if !canvas_rect.intersects(rect) {
                continue;
            }

            let is_selected = self.selected_node == Some(idx);
            let fill = if is_selected {
                Color32::from_rgb(60, 60, 80)
            } else {
                Color32::from_rgb(45, 45, 58)
            };
            let border_color = if is_selected {
                Color32::from_rgb(140, 180, 255)
            } else {
                Color32::from_rgb(100, 100, 130)
            };

            painter.rect_filled(rect, 6.0, fill);
            painter.rect_stroke(
                rect,
                6.0,
                egui::Stroke::new(if is_selected { 2.0 } else { 1.5 }, border_color),
                egui::StrokeKind::Middle,
            );
            painter.text(
                rect.min + egui::vec2(8.0, 6.0),
                egui::Align2::LEFT_TOP,
                &node.label,
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::WHITE,
            );

            // Output port
            if !node.output.is_empty() {
                let port_pos = output_port_pos(rect);
                let port_color = if is_wiring {
                    Color32::from_rgb(160, 240, 160)
                } else {
                    Color32::from_rgb(120, 200, 120)
                };
                painter.circle_filled(port_pos, 5.0 * self.zoom, port_color);
                painter.text(
                    port_pos + egui::vec2(-6.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &node.output,
                    egui::FontId::proportional(9.0 * self.zoom),
                    Color32::from_rgb(180, 210, 180),
                );
            }

            // Input ports
            let num = node.inputs.len();
            for (i, input_name) in node.inputs.iter().enumerate() {
                let port_pos = input_port_pos(rect, i, num);
                let port_color = if is_wiring {
                    Color32::from_rgb(240, 210, 100)
                } else {
                    Color32::from_rgb(200, 140, 80)
                };
                painter.circle_filled(port_pos, 5.0 * self.zoom, port_color);
                painter.text(
                    port_pos + egui::vec2(6.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    input_name,
                    egui::FontId::proportional(9.0 * self.zoom),
                    Color32::from_rgb(210, 190, 160),
                );
            }

            // Click-to-select (only when not drawing a wire).
            if !is_wiring {
                let node_resp =
                    ui.interact(rect, ui.id().with(("mat_node", idx)), egui::Sense::click());
                if node_resp.clicked() {
                    new_selected = Some(idx);
                }
            }
        }
        self.selected_node = new_selected;

        painter.text(
            canvas_rect.left_bottom() + egui::vec2(8.0, -12.0),
            egui::Align2::LEFT_BOTTOM,
            "Drag output port → input port to wire  •  scroll to zoom  •  drag canvas to pan  •  drag node to move",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(70, 70, 90),
        );
    }
}
