//! Visual Logic (node graph) Editor panel for NovaForge Workspace.
//!
//! The node-graph layer is intentionally shared with the Material Editor's
//! canvas pattern.  A dedicated blueprint / behaviour graph will be integrated
//! in a later phase.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Hit-test radius for port circles, in screen pixels (zoom-independent).
const PORT_RADIUS: f32 = 9.0;

// ---------------------------------------------------------------------------
// DragMode
// ---------------------------------------------------------------------------

/// What the canvas drag gesture is currently doing.
#[derive(Default, Clone, Copy, Debug)]
enum DragMode {
    #[default]
    Idle,
    /// Dragging a specific node.
    MovingNode(usize),
    /// Panning the canvas.
    PanningCanvas,
    /// Drawing an edge from a node's output port.
    DrawingEdge {
        from_node: usize,
        /// Screen-space origin of the tentative edge.
        from_pos: egui::Pos2,
    },
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

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

/// Output port position for a node rect (right edge, vertically centred).
fn output_port(rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(rect.right(), rect.center().y)
}

/// Input port position for a node rect (left edge, vertically centred).
fn input_port(rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(rect.left(), rect.center().y)
}

/// Compute the screen-space rect for a logic node.
fn vlogic_node_rect(
    canvas_rect: egui::Rect,
    pan: egui::Vec2,
    zoom: f32,
    node: &LogicNode,
    node_w: f32,
    node_h: f32,
) -> egui::Rect {
    let top_left = canvas_rect.min + pan + node.pos.to_vec2() * zoom;
    egui::Rect::from_min_size(top_left, egui::vec2(node_w, node_h))
}

// ---------------------------------------------------------------------------
// LogicNode
// ---------------------------------------------------------------------------

/// A logic node in the visual scripting graph.
#[derive(Clone)]
struct LogicNode {
    #[allow(dead_code)]
    id: usize,
    label: String,
    pos: egui::Pos2,
    colour: Color32,
}

// ---------------------------------------------------------------------------
// VLogicEditor
// ---------------------------------------------------------------------------

/// Visual Logic Editor panel.
///
/// Provides a blueprint-style node graph.  Drag from the right side of a node
/// to the left side of another to draw an edge.  Scroll to zoom; drag the
/// canvas to pan.
///
/// The graph can be saved to and loaded from `<asset_root>/logic/logic_graph.toml`.
pub struct VLogicEditor {
    nodes: Vec<LogicNode>,
    zoom: f32,
    pan: egui::Vec2,
    selected_node: Option<usize>,
    /// Next unique node ID.
    next_id: usize,
    /// Current canvas drag mode.
    drag_mode: DragMode,
    /// Directed edges: (from_node_index, to_node_index).
    edges: Vec<(usize, usize)>,
    /// Status message shown in the toolbar (save/load feedback).
    save_status: String,
}

impl Default for VLogicEditor {
    fn default() -> Self {
        Self {
            nodes: vec![
                LogicNode {
                    id: 0,
                    label: "On Player Enter".to_string(),
                    pos: egui::pos2(60.0, 80.0),
                    colour: Color32::from_rgb(60, 100, 160),
                },
                LogicNode {
                    id: 1,
                    label: "Branch".to_string(),
                    pos: egui::pos2(260.0, 90.0),
                    colour: Color32::from_rgb(80, 80, 80),
                },
                LogicNode {
                    id: 2,
                    label: "Spawn Effect".to_string(),
                    pos: egui::pos2(440.0, 60.0),
                    colour: Color32::from_rgb(60, 140, 80),
                },
                LogicNode {
                    id: 3,
                    label: "Play Sound".to_string(),
                    pos: egui::pos2(440.0, 160.0),
                    colour: Color32::from_rgb(140, 80, 60),
                },
            ],
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            selected_node: None,
            next_id: 4,
            drag_mode: DragMode::Idle,
            // Default edges matching the original hard-coded demonstration.
            // Edge indices are validated with `.get()` during drawing, so
            // stale entries after node deletion are silently skipped.
            edges: vec![(0, 1), (1, 2), (1, 3)],
            save_status: String::new(),
        }
    }
}

impl EditorPanel for VLogicEditor {
    fn title(&self) -> &str {
        "Visual Logic"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        // ── Toolbar ──────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.button("＋ Event Node").clicked() {
                let id = self.next_id;
                self.next_id += 1;
                let offset = egui::vec2(10.0, 10.0) * id as f32;
                self.nodes.push(LogicNode {
                    id,
                    label: format!("Event {id}"),
                    pos: egui::pos2(60.0, 60.0) + offset,
                    colour: Color32::from_rgb(60, 100, 160),
                });
                self.selected_node = Some(self.nodes.len() - 1);
            }
            if ui.button("＋ Action Node").clicked() {
                let id = self.next_id;
                self.next_id += 1;
                let offset = egui::vec2(10.0, 10.0) * id as f32;
                self.nodes.push(LogicNode {
                    id,
                    label: format!("Action {id}"),
                    pos: egui::pos2(240.0, 60.0) + offset,
                    colour: Color32::from_rgb(60, 140, 80),
                });
                self.selected_node = Some(self.nodes.len() - 1);
            }
            if ui.button("＋ Branch").clicked() {
                let id = self.next_id;
                self.next_id += 1;
                let offset = egui::vec2(10.0, 10.0) * id as f32;
                self.nodes.push(LogicNode {
                    id,
                    label: format!("Branch {id}"),
                    pos: egui::pos2(150.0, 100.0) + offset,
                    colour: Color32::from_rgb(80, 80, 80),
                });
                self.selected_node = Some(self.nodes.len() - 1);
            }
            let delete_enabled = self.selected_node.is_some();
            if ui
                .add_enabled(delete_enabled, egui::Button::new("🗑 Delete"))
                .clicked()
            {
                if let Some(idx) = self.selected_node {
                    self.nodes.remove(idx);
                    // Prune edges touching the deleted node and fix indices.
                    self.edges.retain(|&(from, to)| from != idx && to != idx);
                    for (from, to) in &mut self.edges {
                        if *from > idx {
                            *from -= 1;
                        }
                        if *to > idx {
                            *to -= 1;
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
            if ui
                .button("💾 Save")
                .on_hover_text("Save graph to <asset_root>/logic/logic_graph.toml")
                .clicked()
            {
                self.save_graph(ctx);
            }
            if ui
                .button("📂 Load")
                .on_hover_text("Load graph from <asset_root>/logic/logic_graph.toml")
                .clicked()
            {
                self.load_graph(ctx);
            }
            ui.separator();
            if ui.button("🔍＋").clicked() {
                self.zoom = (self.zoom + 0.1).min(3.0);
            }
            if ui.button("🔍−").clicked() {
                self.zoom = (self.zoom - 0.1).max(0.3);
            }
            if ui.button("⊙ Reset").clicked() {
                self.zoom = 1.0;
                self.pan = egui::Vec2::ZERO;
            }
            ui.separator();
            if ui
                .button("🗑 Clear Edges")
                .on_hover_text("Remove all edge connections")
                .clicked()
            {
                self.edges.clear();
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
            if matches!(self.drag_mode, DragMode::DrawingEdge { .. }) {
                ui.separator();
                ui.label(
                    egui::RichText::new("🔌 Drawing edge — release on another node to connect")
                        .size(11.0)
                        .color(Color32::from_rgb(220, 210, 80)),
                );
            }
        });

        // Status line (save / load feedback).
        if !self.save_status.is_empty() {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&self.save_status)
                        .size(11.0)
                        .color(Color32::from_rgb(160, 200, 160)),
                );
            });
        }

        ui.separator();

        // ── Canvas allocation ────────────────────────────────────────────────
        let available = ui.available_size();
        let (canvas_rect, response) =
            ui.allocate_exact_size(available, egui::Sense::click_and_drag());

        let node_w = 130.0 * self.zoom;
        let node_h = 32.0 * self.zoom;

        // Scroll-wheel zoom.
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

                // Priority 1: output port (right edge) → start edge drawing.
                'port: for (idx, node) in self.nodes.iter().enumerate() {
                    let rect =
                        vlogic_node_rect(canvas_rect, self.pan, self.zoom, node, node_w, node_h);
                    let port_pos = output_port(rect);
                    if pos.distance(port_pos) < PORT_RADIUS {
                        new_mode = DragMode::DrawingEdge {
                            from_node: idx,
                            from_pos: port_pos,
                        };
                        break 'port;
                    }
                }

                // Priority 2: node body → move node.
                if matches!(new_mode, DragMode::PanningCanvas) {
                    for (idx, node) in self.nodes.iter().enumerate() {
                        let rect = vlogic_node_rect(
                            canvas_rect,
                            self.pan,
                            self.zoom,
                            node,
                            node_w,
                            node_h,
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
                DragMode::DrawingEdge { .. } | DragMode::Idle => {}
            }
        }

        if response.drag_stopped() {
            if let DragMode::DrawingEdge { from_node, .. } = self.drag_mode {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    for (to_idx, node) in self.nodes.iter().enumerate() {
                        if to_idx == from_node {
                            continue;
                        }
                        let rect = vlogic_node_rect(
                            canvas_rect,
                            self.pan,
                            self.zoom,
                            node,
                            node_w,
                            node_h,
                        );
                        let port_pos = input_port(rect);
                        if pos.distance(port_pos) < PORT_RADIUS {
                            // Avoid duplicate edges.
                            if !self.edges.contains(&(from_node, to_idx)) {
                                self.edges.push((from_node, to_idx));
                            }
                            break;
                        }
                    }
                }
            }
            self.drag_mode = DragMode::Idle;
        }

        // Recompute node_w/node_h after potential zoom change.
        let node_w = 130.0 * self.zoom;
        let node_h = 32.0 * self.zoom;

        // ── Painting ─────────────────────────────────────────────────────────
        let painter = ui.painter_at(canvas_rect);
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(18, 18, 24));

        // Grid
        let step = 28.0 * self.zoom;
        let ox = (canvas_rect.min + self.pan).x % step;
        let oy = (canvas_rect.min + self.pan).y % step;
        let mut x = canvas_rect.left() + ox;
        while x < canvas_rect.right() {
            let mut y = canvas_rect.top() + oy;
            while y < canvas_rect.bottom() {
                painter.circle_filled(egui::pos2(x, y), 1.0, Color32::from_rgb(40, 40, 52));
                y += step;
            }
            x += step;
        }

        // ── Edges ────────────────────────────────────────────────────────────
        for &(from, to) in &self.edges {
            let from_rect = self
                .nodes
                .get(from)
                .map(|n| vlogic_node_rect(canvas_rect, self.pan, self.zoom, n, node_w, node_h));
            let to_rect = self
                .nodes
                .get(to)
                .map(|n| vlogic_node_rect(canvas_rect, self.pan, self.zoom, n, node_w, node_h));
            if let (Some(fr), Some(tr)) = (from_rect, to_rect) {
                let p0 = output_port(fr);
                let p3 = input_port(tr);
                let ctrl = ((p3.x - p0.x).abs() * 0.5).max(50.0);
                let p1 = egui::pos2(p0.x + ctrl, p0.y);
                let p2 = egui::pos2(p3.x - ctrl, p3.y);
                draw_bezier(
                    &painter,
                    p0,
                    p1,
                    p2,
                    p3,
                    Color32::from_rgb(130, 130, 165),
                    2.0 * self.zoom,
                );
            }
        }

        // ── In-progress edge ─────────────────────────────────────────────────
        if let DragMode::DrawingEdge { from_pos, .. } = self.drag_mode {
            if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                let ctrl = ((cursor.x - from_pos.x).abs() * 0.5).max(50.0);
                let p1 = egui::pos2(from_pos.x + ctrl, from_pos.y);
                let p2 = egui::pos2(cursor.x - ctrl, cursor.y);
                draw_bezier(
                    &painter,
                    from_pos,
                    p1,
                    p2,
                    cursor,
                    Color32::from_rgb(230, 220, 80),
                    2.0,
                );
                ui.ctx().request_repaint();
            }
        }

        // ── Nodes ────────────────────────────────────────────────────────────
        let is_wiring = matches!(self.drag_mode, DragMode::DrawingEdge { .. });
        let mut new_selected = self.selected_node;
        for (idx, node) in self.nodes.iter().enumerate() {
            let rect = vlogic_node_rect(canvas_rect, self.pan, self.zoom, node, node_w, node_h);
            if !canvas_rect.intersects(rect) {
                continue;
            }
            let is_selected = self.selected_node == Some(idx);
            painter.rect_filled(rect, 4.0, node.colour);
            painter.rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(
                    if is_selected { 2.5 } else { 1.0 },
                    if is_selected {
                        Color32::from_rgb(220, 220, 255)
                    } else {
                        Color32::from_rgb(200, 200, 220)
                    },
                ),
                egui::StrokeKind::Middle,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &node.label,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::WHITE,
            );

            // Port circles (highlighted when wiring).
            let out_color = if is_wiring {
                Color32::from_rgb(160, 240, 160)
            } else {
                Color32::from_rgb(120, 200, 120)
            };
            let in_color = if is_wiring {
                Color32::from_rgb(240, 210, 100)
            } else {
                Color32::from_rgb(200, 160, 80)
            };
            painter.circle_filled(output_port(rect), 4.0 * self.zoom, out_color);
            painter.circle_filled(input_port(rect), 4.0 * self.zoom, in_color);

            // Click-to-select.
            if !is_wiring {
                let node_response = ui.interact(
                    rect,
                    ui.id().with(("vlogic_node", idx)),
                    egui::Sense::click(),
                );
                if node_response.clicked() {
                    new_selected = Some(idx);
                }
            }
        }
        self.selected_node = new_selected;

        painter.text(
            canvas_rect.left_bottom() + egui::vec2(8.0, -12.0),
            egui::Align2::LEFT_BOTTOM,
            "Drag output port → input port to connect  •  scroll to zoom  •  drag canvas to pan  •  drag node to move",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(70, 70, 90),
        );
    }
}

// ---------------------------------------------------------------------------
// Save / load helpers
// ---------------------------------------------------------------------------

/// Serialisable form of a [`LogicNode`] colour (RGBA bytes).
#[derive(Serialize, Deserialize)]
struct NodeColour {
    r: u8,
    g: u8,
    b: u8,
}

/// Serialisable form of a [`LogicNode`].
#[derive(Serialize, Deserialize)]
struct LogicNodeFile {
    id: usize,
    label: String,
    pos: [f32; 2],
    colour: NodeColour,
}

/// Root structure for `logic_graph.toml`.
#[derive(Serialize, Deserialize)]
struct LogicGraphFile {
    next_id: usize,
    nodes: Vec<LogicNodeFile>,
    edges: Vec<[usize; 2]>,
}

impl VLogicEditor {
    fn graph_path(ctx: &PanelContext) -> Option<PathBuf> {
        ctx.asset_root
            .as_ref()
            .map(|r| r.join("logic").join("logic_graph.toml"))
    }

    fn save_graph(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::graph_path(ctx) else {
            self.save_status = "No project loaded — cannot save graph.".to_string();
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.save_status = format!("Directory error: {e}");
                return;
            }
        }
        let file = LogicGraphFile {
            next_id: self.next_id,
            nodes: self
                .nodes
                .iter()
                .map(|n| {
                    let [r, g, b, _] = n.colour.to_array();
                    LogicNodeFile {
                        id: n.id,
                        label: n.label.clone(),
                        pos: [n.pos.x, n.pos.y],
                        colour: NodeColour { r, g, b },
                    }
                })
                .collect(),
            edges: self.edges.iter().map(|&(f, t)| [f, t]).collect(),
        };
        match toml::to_string_pretty(&file) {
            Ok(content) => match std::fs::write(&path, content) {
                Ok(()) => self.save_status = format!("Saved → {}", path.display()),
                Err(e) => self.save_status = format!("Write error: {e}"),
            },
            Err(e) => self.save_status = format!("Serialise error: {e}"),
        }
    }

    fn load_graph(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::graph_path(ctx) else {
            self.save_status = "No project loaded — cannot load graph.".to_string();
            return;
        };
        if !path.exists() {
            self.save_status = format!("File not found: {}", path.display());
            return;
        }
        match std::fs::read_to_string(&path) {
            Err(e) => self.save_status = format!("Read error: {e}"),
            Ok(content) => match toml::from_str::<LogicGraphFile>(&content) {
                Err(e) => self.save_status = format!("Parse error: {e}"),
                Ok(file) => {
                    self.next_id = file.next_id;
                    self.nodes = file
                        .nodes
                        .into_iter()
                        .map(|n| LogicNode {
                            id: n.id,
                            label: n.label,
                            pos: egui::pos2(n.pos[0], n.pos[1]),
                            colour: Color32::from_rgb(n.colour.r, n.colour.g, n.colour.b),
                        })
                        .collect();
                    self.edges = file.edges.into_iter().map(|e| (e[0], e[1])).collect();
                    self.selected_node = None;
                    self.save_status =
                        format!("Loaded {} nodes ← {}", self.nodes.len(), path.display());
                }
            },
        }
    }
}
