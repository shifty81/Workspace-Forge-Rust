//! Visual Logic (node graph) Editor panel for NovaForge Workspace.
//!
//! The node-graph layer is intentionally shared with the Material Editor's
//! canvas pattern.  A dedicated blueprint / behaviour graph will be integrated
//! in a later phase.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// A logic node in the visual scripting graph.
#[derive(Clone)]
struct LogicNode {
    #[allow(dead_code)]
    id: usize,
    label: String,
    pos: egui::Pos2,
    colour: Color32,
}

/// Visual Logic Editor panel.
pub struct VLogicEditor {
    nodes: Vec<LogicNode>,
    zoom: f32,
    pan: egui::Vec2,
    selected_node: Option<usize>,
    /// Next unique node ID.
    next_id: usize,
    /// Index of the node currently being dragged, if any.
    dragging_node: Option<usize>,
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
            dragging_node: None,
        }
    }
}

impl EditorPanel for VLogicEditor {
    fn title(&self) -> &str {
        "Visual Logic"
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
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
                    self.selected_node = if self.nodes.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.nodes.len() - 1))
                    };
                }
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
        });

        ui.separator();

        let available = ui.available_size();
        let (canvas_rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());

        // Node dimensions (needed before drag handling).
        let node_w = 130.0 * self.zoom;
        let node_h = 32.0 * self.zoom;

        // ── Drag handling: node drag vs. canvas pan ───────────────────────────
        if response.drag_started() {
            self.dragging_node = None;
            if let Some(pos) = response.interact_pointer_pos() {
                for (idx, node) in self.nodes.iter().enumerate() {
                    let top_left = canvas_rect.min + self.pan + node.pos.to_vec2() * self.zoom;
                    let rect = egui::Rect::from_min_size(top_left, egui::vec2(node_w, node_h));
                    if rect.contains(pos) {
                        self.dragging_node = Some(idx);
                        self.selected_node = Some(idx);
                        break;
                    }
                }
            }
        }
        if response.dragged() {
            let delta = response.drag_delta();
            if let Some(idx) = self.dragging_node {
                if let Some(node) = self.nodes.get_mut(idx) {
                    node.pos += delta / self.zoom;
                }
            } else {
                self.pan += delta;
            }
        }
        if response.drag_stopped() {
            self.dragging_node = None;
        }

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

        // Edges (stub connections as straight lines between node centres)
        let node_positions: Vec<egui::Pos2> = self
            .nodes
            .iter()
            .map(|n| {
                canvas_rect.min
                    + self.pan
                    + n.pos.to_vec2() * self.zoom
                    + egui::vec2(node_w * 0.5, node_h * 0.5)
            })
            .collect();

        // Hard-coded edges for the default nodes (0→1, 1→2, 1→3); skip if nodes
        // no longer exist (they might have been deleted).
        for (from, to) in [(0usize, 1usize), (1, 2), (1, 3)] {
            if let (Some(&p0), Some(&p1)) = (node_positions.get(from), node_positions.get(to)) {
                if canvas_rect.contains(p0) || canvas_rect.contains(p1) {
                    painter.line_segment(
                        [p0, p1],
                        egui::Stroke::new(2.0 * self.zoom, Color32::from_rgb(130, 130, 160)),
                    );
                }
            }
        }

        // Nodes
        let mut new_selected = self.selected_node;
        for (idx, node) in self.nodes.iter().enumerate() {
            let top_left = canvas_rect.min + self.pan + node.pos.to_vec2() * self.zoom;
            let rect = egui::Rect::from_min_size(top_left, egui::vec2(node_w, node_h));
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

            // Click-to-select
            let node_response =
                ui.interact(rect, ui.id().with(("vlogic_node", idx)), egui::Sense::click());
            if node_response.clicked() {
                new_selected = Some(idx);
            }
        }
        self.selected_node = new_selected;

        painter.text(
            canvas_rect.left_bottom() + egui::vec2(8.0, -12.0),
            egui::Align2::LEFT_BOTTOM,
            "Visual Logic graph — drag canvas to pan  •  drag a node to move it  •  click a node to select",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(70, 70, 90),
        );
    }
}
