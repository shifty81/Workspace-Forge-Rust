//! Material / Shader Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// A node in the material graph.
#[derive(Clone)]
#[allow(dead_code)]
struct MaterialNode {
    id: usize,
    label: String,
    pos: egui::Pos2,
    inputs: Vec<String>,
    output: String,
}

/// Material / Shader Editor panel.
///
/// Displays a node-graph canvas placeholder.  Full egui_node_graph integration
/// will be wired in during Phase 2 of the editor development.
pub struct MaterialEditor {
    nodes: Vec<MaterialNode>,
    zoom: f32,
    pan: egui::Vec2,
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
        }
    }
}

impl EditorPanel for MaterialEditor {
    fn title(&self) -> &str {
        "Material Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("＋ Add Node").clicked() {
                // TODO: add node dialog
            }
            if ui.button("🗑 Delete Node").clicked() {
                // TODO: delete selected node
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
        });

        ui.separator();

        // Node graph canvas (placeholder rendering)
        let available = ui.available_size();
        let (canvas_rect, response) =
            ui.allocate_exact_size(available, egui::Sense::drag());

        if response.dragged() {
            self.pan += response.drag_delta();
        }

        let painter = ui.painter_at(canvas_rect);
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(20, 20, 26));

        // Dot-grid background
        let step = 24.0 * self.zoom;
        let origin = canvas_rect.min + self.pan;
        let offset_x = origin.x % step;
        let offset_y = origin.y % step;
        let mut x = canvas_rect.left() + offset_x;
        while x < canvas_rect.right() {
            let mut y = canvas_rect.top() + offset_y;
            while y < canvas_rect.bottom() {
                painter.circle_filled(
                    egui::pos2(x, y),
                    1.0,
                    Color32::from_rgb(50, 50, 62),
                );
                y += step;
            }
            x += step;
        }

        // Draw nodes
        let node_w = 120.0 * self.zoom;
        let node_h = 64.0 * self.zoom;
        for node in &self.nodes {
            let top_left = canvas_rect.min
                + self.pan
                + node.pos.to_vec2() * self.zoom;
            let rect = egui::Rect::from_min_size(top_left, egui::vec2(node_w, node_h));

            if !canvas_rect.intersects(rect) {
                continue;
            }

            painter.rect_filled(rect, 6.0, Color32::from_rgb(45, 45, 58));
            painter.rect_stroke(
                rect,
                6.0,
                egui::Stroke::new(1.5, Color32::from_rgb(100, 100, 130)),
                egui::StrokeKind::Middle,
            );
            painter.text(
                rect.min + egui::vec2(8.0, 6.0),
                egui::Align2::LEFT_TOP,
                &node.label,
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::WHITE,
            );
            // Output port stub
            if !node.output.is_empty() {
                let port = egui::pos2(rect.right(), rect.center().y);
                painter.circle_filled(port, 5.0 * self.zoom, Color32::from_rgb(120, 200, 120));
            }
            // Input port stubs
            for (i, _input) in node.inputs.iter().enumerate() {
                let y = rect.min.y + (i as f32 + 1.0) * node_h / (node.inputs.len() as f32 + 1.0);
                let port = egui::pos2(rect.left(), y);
                painter.circle_filled(port, 5.0 * self.zoom, Color32::from_rgb(200, 140, 80));
            }
        }

        painter.text(
            canvas_rect.center() + egui::vec2(0.0, canvas_rect.height() * 0.35),
            egui::Align2::CENTER_CENTER,
            "Node Graph — drag to pan  •  full egui_node_graph integration pending",
            egui::FontId::proportional(11.0),
            Color32::from_rgb(70, 70, 90),
        );
    }
}
