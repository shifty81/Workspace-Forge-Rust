//! Visual Logic (node graph) Editor panel for NovaForge Workspace.
//!
//! The node-graph layer is intentionally shared with the Material Editor's
//! canvas pattern.  A dedicated blueprint / behaviour graph will be integrated
//! in a later phase.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// A logic node in the visual scripting graph.
#[derive(Clone)]
#[allow(dead_code)]
struct LogicNode {
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
                // TODO: add event node
            }
            if ui.button("＋ Action Node").clicked() {
                // TODO: add action node
            }
            if ui.button("＋ Branch").clicked() {
                // TODO: add branch node
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
        });

        ui.separator();

        let available = ui.available_size();
        let (canvas_rect, response) =
            ui.allocate_exact_size(available, egui::Sense::drag());

        if response.dragged() {
            self.pan += response.drag_delta();
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
            .map(|n| canvas_rect.min + self.pan + n.pos.to_vec2() * self.zoom + egui::vec2(60.0 * self.zoom, 16.0 * self.zoom))
            .collect();

        // Hard-coded edges: 0→1, 1→2, 1→3
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
        let node_w = 130.0 * self.zoom;
        let node_h = 32.0 * self.zoom;
        for node in &self.nodes {
            let top_left = canvas_rect.min + self.pan + node.pos.to_vec2() * self.zoom;
            let rect = egui::Rect::from_min_size(top_left, egui::vec2(node_w, node_h));
            if !canvas_rect.intersects(rect) {
                continue;
            }
            painter.rect_filled(rect, 4.0, node.colour);
            painter.rect_stroke(
                rect,
                4.0,
                egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 220)),
                egui::StrokeKind::Middle,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &node.label,
                egui::FontId::proportional(11.0 * self.zoom),
                Color32::WHITE,
            );
        }

        painter.text(
            canvas_rect.left_bottom() + egui::vec2(8.0, -12.0),
            egui::Align2::LEFT_BOTTOM,
            "Visual Logic graph — drag to pan  •  full integration pending",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(70, 70, 90),
        );
    }
}
