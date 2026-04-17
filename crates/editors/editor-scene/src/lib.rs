//! Scene / World Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// Gizmo / transform mode selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// Scene Editor panel.
///
/// Displays a 3-D viewport placeholder and a basic entity inspector.
/// Full rendering will be wired in when Nova-Forge's render pipeline is
/// integrated.
#[derive(Default)]
pub struct SceneEditor {
    gizmo_mode: GizmoMode,
    selected_entity: Option<String>,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
    entity_name_buf: String,
}

impl SceneEditor {
    pub fn new() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0],
            ..Default::default()
        }
    }
}

impl EditorPanel for SceneEditor {
    fn title(&self) -> &str {
        "Scene Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Translate, "⬆ Translate");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Rotate, "↻ Rotate");
            ui.selectable_value(&mut self.gizmo_mode, GizmoMode::Scale, "⤢ Scale");
            ui.separator();
            if ui.button("＋ Entity").clicked() {
                self.selected_entity = Some("New Entity".to_string());
                self.entity_name_buf = "New Entity".to_string();
                self.position = [0.0; 3];
                self.rotation = [0.0; 3];
                self.scale = [1.0; 3];
            }
            if ui.button("🗑 Delete").clicked() {
                self.selected_entity = None;
            }
        });

        ui.separator();

        // Viewport (placeholder)
        let available = ui.available_size();
        let viewport_height = (available.y - 96.0).max(80.0);
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(available.x, viewport_height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, Color32::from_rgb(22, 22, 28));
        painter.rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, Color32::from_rgb(55, 55, 68)),
            egui::StrokeKind::Middle,
        );
        // Grid lines hint
        for i in 1..8 {
            let x = rect.left() + rect.width() * (i as f32 / 8.0);
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(0.5, Color32::from_rgb(40, 40, 50)),
            );
        }
        for i in 1..6 {
            let y = rect.top() + rect.height() * (i as f32 / 6.0);
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5, Color32::from_rgb(40, 40, 50)),
            );
        }
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "🌐  3-D Viewport\nRendering integration pending",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(90, 90, 115),
        );

        // Inspector
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Entity:");
            match &self.selected_entity {
                Some(name) => {
                    ui.strong(name);
                }
                None => {
                    ui.label(egui::RichText::new("None").italics());
                }
            }
        });

        egui::Grid::new("transform_grid")
            .num_columns(4)
            .spacing([4.0, 2.0])
            .show(ui, |ui| {
                ui.label("Position");
                ui.add(egui::DragValue::new(&mut self.position[0]).prefix("X ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.position[1]).prefix("Y ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.position[2]).prefix("Z ").speed(0.1));
                ui.end_row();

                ui.label("Rotation");
                ui.add(egui::DragValue::new(&mut self.rotation[0]).prefix("X ").speed(0.5));
                ui.add(egui::DragValue::new(&mut self.rotation[1]).prefix("Y ").speed(0.5));
                ui.add(egui::DragValue::new(&mut self.rotation[2]).prefix("Z ").speed(0.5));
                ui.end_row();

                ui.label("Scale   ");
                ui.add(egui::DragValue::new(&mut self.scale[0]).prefix("X ").speed(0.01));
                ui.add(egui::DragValue::new(&mut self.scale[1]).prefix("Y ").speed(0.01));
                ui.add(egui::DragValue::new(&mut self.scale[2]).prefix("Z ").speed(0.01));
                ui.end_row();
            });
    }
}
