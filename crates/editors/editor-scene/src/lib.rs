//! Scene / World Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// Width of the entity list sidebar in pixels.
const ENTITY_LIST_WIDTH: f32 = 160.0;
/// Height reserved for the transform inspector below the viewport.
const INSPECTOR_HEIGHT: f32 = 130.0;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// A single entity in the scene.
#[derive(Clone)]
struct SceneEntity {
    name: String,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

impl SceneEntity {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            position: [0.0; 3],
            rotation: [0.0; 3],
            scale: [1.0; 3],
        }
    }
}

/// Scene Editor panel.
///
/// Displays a 3-D viewport placeholder and a basic entity list / inspector.
/// Full rendering will be wired in when Nova-Forge's render pipeline is
/// integrated.
pub struct SceneEditor {
    gizmo_mode: GizmoMode,
    entities: Vec<SceneEntity>,
    selected: Option<usize>,
    /// Counter used to generate unique default names.
    entity_counter: u32,
}

impl Default for SceneEditor {
    fn default() -> Self {
        Self {
            gizmo_mode: GizmoMode::default(),
            entities: vec![
                SceneEntity::new("World Root"),
                SceneEntity {
                    name: "Player Spawn".to_string(),
                    position: [0.0, 1.0, 0.0],
                    rotation: [0.0; 3],
                    scale: [1.0; 3],
                },
            ],
            selected: None,
            entity_counter: 2,
        }
    }
}

impl SceneEditor {
    pub fn new() -> Self {
        Self::default()
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
                self.entity_counter += 1;
                let name = format!("Entity {}", self.entity_counter);
                self.entities.push(SceneEntity::new(name));
                self.selected = Some(self.entities.len() - 1);
            }
            let delete_enabled = self.selected.is_some();
            if ui
                .add_enabled(delete_enabled, egui::Button::new("🗑 Delete"))
                .clicked()
            {
                if let Some(idx) = self.selected {
                    self.entities.remove(idx);
                    // Select the entity at the same index (now pointing to what
                    // was next), or the last one if we deleted the last entry.
                    self.selected = if self.entities.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.entities.len() - 1))
                    };
                }
            }
        });

        ui.separator();

        // Split the remaining area: left = entity list, right = viewport + inspector.
        let available = ui.available_size();

        ui.horizontal(|ui| {
            // ── Entity List ──────────────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_width(ENTITY_LIST_WIDTH);
                ui.strong("Entities");
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("entity_list")
                    .max_height(available.y - 32.0)                    .show(ui, |ui| {
                        let mut new_selected = self.selected;
                        for (i, entity) in self.entities.iter().enumerate() {
                            let selected = self.selected == Some(i);
                            if ui
                                .selectable_label(selected, format!("🔷 {}", entity.name))
                                .clicked()
                            {
                                new_selected = Some(i);
                            }
                        }
                        if self.entities.is_empty() {
                            ui.label(
                                egui::RichText::new("No entities.\nPress ＋ to add one.")
                                    .italics()
                                    .color(Color32::from_rgb(120, 120, 140)),
                            );
                        }
                        self.selected = new_selected;
                    });
            });

            ui.separator();

            // ── Viewport + Inspector ──────────────────────────────────────────
            ui.vertical(|ui| {
                let right_w = available.x - ENTITY_LIST_WIDTH - 8.0;
                let viewport_h = (available.y - INSPECTOR_HEIGHT - 12.0).max(60.0);

                // Viewport placeholder
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(right_w, viewport_h),
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
                if let Some(idx) = self.selected {
                    if let Some(entity) = self.entities.get_mut(idx) {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut entity.name);
                        });

                        egui::Grid::new("transform_grid")
                            .num_columns(4)
                            .spacing([4.0, 2.0])
                            .show(ui, |ui| {
                                ui.label("Position");
                                for (prefix, v) in
                                    ["X ", "Y ", "Z "].iter().zip(entity.position.iter_mut())
                                {
                                    ui.add(
                                        egui::DragValue::new(v).prefix(*prefix).speed(0.1),
                                    );
                                }
                                ui.end_row();

                                ui.label("Rotation");
                                for (prefix, v) in
                                    ["X ", "Y ", "Z "].iter().zip(entity.rotation.iter_mut())
                                {
                                    ui.add(
                                        egui::DragValue::new(v).prefix(*prefix).speed(0.5),
                                    );
                                }
                                ui.end_row();

                                ui.label("Scale   ");
                                for (prefix, v) in
                                    ["X ", "Y ", "Z "].iter().zip(entity.scale.iter_mut())
                                {
                                    ui.add(
                                        egui::DragValue::new(v).prefix(*prefix).speed(0.01),
                                    );
                                }
                                ui.end_row();
                            });
                    }
                } else {
                    ui.label(
                        egui::RichText::new("Select an entity to inspect its transform.")
                            .italics()
                            .color(Color32::from_rgb(120, 120, 140)),
                    );
                }
            });
        });
    }
}
