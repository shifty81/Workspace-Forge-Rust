//! Scene / World Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::path::PathBuf;

/// Width of the entity list sidebar in pixels.
const ENTITY_LIST_WIDTH: f32 = 160.0;
/// Height reserved for the transform inspector below the viewport.
const INSPECTOR_HEIGHT: f32 = 130.0;

// ---------------------------------------------------------------------------
// Pie menu constants
// ---------------------------------------------------------------------------

/// Outer radius of the pie menu (px).
const PIE_RADIUS: f32 = 88.0;
/// Inner dead-zone radius — hovering here does not activate any slice.
const PIE_INNER_R: f32 = 22.0;
/// Number of slices.
const PIE_N: usize = 6;
/// (icon, label) pairs for each slice, arranged clockwise from the top.
const PIE_ITEMS: [(&str, &str); PIE_N] = [
    ("⬆", "Translate"),
    ("↻", "Rotate"),
    ("⤢", "Scale"),
    ("＋", "Entity"),
    ("⧉", "Dup"),
    ("🗑", "Delete"),
];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// A single entity in the scene.
#[derive(Clone, Serialize, Deserialize)]
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

/// Serialisable wrapper used when writing / reading the scene file.
#[derive(Serialize, Deserialize)]
struct SceneFile {
    entities: Vec<SceneEntity>,
}

/// Scene Editor panel.
///
/// Displays a 3-D viewport placeholder and a basic entity list / inspector.
/// Entities can be saved to and loaded from `<asset_root>/scenes/scene.toml`.
/// Full rendering will be wired in when Nova-Forge's render pipeline is
/// integrated.
pub struct SceneEditor {
    gizmo_mode: GizmoMode,
    entities: Vec<SceneEntity>,
    selected: Option<usize>,
    /// Counter used to generate unique default names.
    entity_counter: u32,
    /// Status message shown below the toolbar (save/load feedback).
    scene_status: String,
    /// When `Some`, the pie menu is displayed at this viewport position.
    pie_menu_pos: Option<egui::Pos2>,
    /// Index of the pie slice currently under the pointer, if any.
    pie_hovered: Option<usize>,
    /// Text typed in the entity-list search box.
    entity_filter: String,
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
            scene_status: String::new(),
            pie_menu_pos: None,
            pie_hovered: None,
            entity_filter: String::new(),
        }
    }
}

impl SceneEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the canonical scene file path derived from the project context.
    fn scene_path(ctx: &PanelContext) -> Option<PathBuf> {
        ctx.asset_root
            .as_ref()
            .map(|r| r.join("scenes").join("scene.toml"))
    }

    // -----------------------------------------------------------------------
    // Pie menu helpers
    // -----------------------------------------------------------------------

    /// Return the slice index (0..PIE_N) that the pointer is hovering, or
    /// `None` when the pointer is in the dead-zone or outside the menu.
    fn pie_slice_at(center: egui::Pos2, pos: egui::Pos2) -> Option<usize> {
        let delta = pos - center;
        let dist = delta.length();
        if !(PIE_INNER_R..=PIE_RADIUS + 6.0).contains(&dist) {
            return None;
        }
        // Angle measured from the top (−π/2) going clockwise.
        let angle = delta.y.atan2(delta.x) + PI / 2.0;
        let normalized = if angle < 0.0 { angle + 2.0 * PI } else { angle };
        let slice = 2.0 * PI / PIE_N as f32;
        Some((normalized / slice) as usize % PIE_N)
    }

    /// Execute the action corresponding to pie slice `idx`.
    fn execute_pie_action(&mut self, idx: usize) {
        match idx {
            0 => self.gizmo_mode = GizmoMode::Translate,
            1 => self.gizmo_mode = GizmoMode::Rotate,
            2 => self.gizmo_mode = GizmoMode::Scale,
            3 => {
                // Add entity
                self.entity_counter += 1;
                let name = format!("Entity {}", self.entity_counter);
                self.entities.push(SceneEntity::new(name));
                self.selected = Some(self.entities.len() - 1);
            }
            4 => {
                // Duplicate selected
                if let Some(idx) = self.selected {
                    if let Some(original) = self.entities.get(idx) {
                        let mut copy = original.clone();
                        copy.name = format!("Copy of {}", original.name);
                        copy.position[0] += 1.0;
                        self.entities.push(copy);
                        self.selected = Some(self.entities.len() - 1);
                    }
                }
            }
            5 => {
                // Delete selected
                if let Some(idx) = self.selected {
                    self.entities.remove(idx);
                    self.selected = if self.entities.is_empty() {
                        None
                    } else {
                        Some(idx.min(self.entities.len() - 1))
                    };
                }
            }
            _ => {}
        }
    }

    /// Draw the pie menu at `center` using `painter`.
    fn draw_pie_menu(
        painter: &egui::Painter,
        center: egui::Pos2,
        hovered: Option<usize>,
        has_selection: bool,
    ) {
        // Outer shadow / backdrop
        painter.circle_filled(
            center,
            PIE_RADIUS + 8.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 160),
        );

        let slice_angle = 2.0 * PI / PIE_N as f32;

        for (i, &(icon, label)) in PIE_ITEMS.iter().enumerate() {
            // Base colour per slice
            let base = pie_slice_color(i, has_selection);
            let is_hov = hovered == Some(i);

            let fill = if is_hov {
                // Brighten on hover
                Color32::from_rgba_premultiplied(
                    (base.r() as u16 + 70).min(255) as u8,
                    (base.g() as u16 + 70).min(255) as u8,
                    (base.b() as u16 + 70).min(255) as u8,
                    230,
                )
            } else {
                Color32::from_rgba_premultiplied(base.r(), base.g(), base.b(), 200)
            };

            // Wedge polygon (convex: 60° arc from center — valid for egui)
            // Start angle measured from the top, clockwise.
            let start_a = i as f32 * slice_angle - PI / 2.0;
            let end_a = start_a + slice_angle;
            const N_SEG: usize = 10;
            let mut pts = Vec::with_capacity(N_SEG + 2);
            pts.push(center);
            for s in 0..=N_SEG {
                let t = s as f32 / N_SEG as f32;
                let a = start_a + (end_a - start_a) * t;
                pts.push(center + egui::vec2(a.cos() * PIE_RADIUS, a.sin() * PIE_RADIUS));
            }

            let stroke_color = if is_hov {
                Color32::WHITE
            } else {
                Color32::from_rgb(60, 60, 75)
            };
            painter.add(egui::Shape::convex_polygon(
                pts,
                fill,
                egui::Stroke::new(if is_hov { 2.0 } else { 1.0 }, stroke_color),
            ));

            // Icon + label inside the slice
            let mid_a = start_a + slice_angle / 2.0;
            let mid_r = (PIE_INNER_R + PIE_RADIUS) / 2.0;
            let label_pos = center + egui::vec2(mid_a.cos() * mid_r, mid_a.sin() * mid_r);

            painter.text(
                label_pos - egui::vec2(0.0, 7.0),
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::proportional(15.0),
                Color32::WHITE,
            );
            painter.text(
                label_pos + egui::vec2(0.0, 8.0),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(8.5),
                if is_hov {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(210, 210, 220)
                },
            );
        }

        // Centre cancel circle
        painter.circle_filled(
            center,
            PIE_INNER_R,
            Color32::from_rgba_premultiplied(25, 25, 38, 240),
        );
        painter.circle_stroke(
            center,
            PIE_INNER_R,
            egui::Stroke::new(1.0, Color32::from_rgb(90, 90, 110)),
        );
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            "✕",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(170, 170, 190),
        );
    }

    fn save_scene(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::scene_path(ctx) else {
            self.scene_status = "No project loaded — cannot save scene.".to_string();
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.scene_status = format!("Directory error: {e}");
                return;
            }
        }
        let file = SceneFile {
            entities: self.entities.clone(),
        };
        match toml::to_string_pretty(&file) {
            Ok(content) => match std::fs::write(&path, content) {
                Ok(()) => {
                    self.scene_status = format!("Saved → {}", path.display());
                }
                Err(e) => {
                    self.scene_status = format!("Write error: {e}");
                }
            },
            Err(e) => {
                self.scene_status = format!("Serialise error: {e}");
            }
        }
    }

    fn load_scene(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::scene_path(ctx) else {
            self.scene_status = "No project loaded — cannot load scene.".to_string();
            return;
        };
        match std::fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.scene_status = format!("File not found: {}", path.display());
            }
            Err(e) => {
                self.scene_status = format!("Read error: {e}");
            }
            Ok(content) => match toml::from_str::<SceneFile>(&content) {
                Ok(file) => {
                    let count = file.entities.len();
                    self.entities = file.entities;
                    self.selected = None;
                    // Set entity_counter to one above the highest numeric suffix
                    // found in loaded entity names (e.g. "Entity 10" → 11), so
                    // that new entities added after loading never duplicate an
                    // existing name.
                    let max_suffix = self
                        .entities
                        .iter()
                        .filter_map(|e| {
                            e.name
                                .strip_prefix("Entity ")
                                .and_then(|s| s.parse::<u32>().ok())
                        })
                        .max()
                        .unwrap_or(0);
                    self.entity_counter = self.entity_counter.max(max_suffix);
                    self.scene_status =
                        format!("Loaded {count} entities ← {}", path.display());
                }
                Err(e) => {
                    self.scene_status = format!("Parse error: {e}");
                }
            },
        }
    }
}

impl EditorPanel for SceneEditor {
    fn title(&self) -> &str {
        "Scene Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
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
            if ui
                .add_enabled(delete_enabled, egui::Button::new("⧉ Duplicate"))
                .on_hover_text("Clone the selected entity")
                .clicked()
            {
                if let Some(idx) = self.selected {
                    if let Some(original) = self.entities.get(idx) {
                        let mut copy = original.clone();
                        copy.name = format!("Copy of {}", original.name);
                        // Offset slightly so the duplicate is visually distinct.
                        copy.position[0] += 1.0;
                        self.entities.push(copy);
                        self.selected = Some(self.entities.len() - 1);
                    }
                }
            }
            ui.separator();
            if ui.button("💾 Save").on_hover_text("Save scene to <asset_root>/scenes/scene.toml").clicked() {
                self.save_scene(ctx);
            }
            if ui.button("📂 Load").on_hover_text("Load scene from <asset_root>/scenes/scene.toml").clicked() {
                self.load_scene(ctx);
            }
        });

        if !self.scene_status.is_empty() {
            ui.label(
                egui::RichText::new(&self.scene_status)
                    .size(11.0)
                    .color(Color32::from_rgb(160, 200, 160)),
            );
        }

        ui.separator();

        // Split the remaining area: left = entity list, right = viewport + inspector.
        let available = ui.available_size();

        ui.horizontal(|ui| {
            // ── Entity List ──────────────────────────────────────────────────
            ui.vertical(|ui| {
                ui.set_width(ENTITY_LIST_WIDTH);
                ui.strong("Entities");
                ui.separator();
                // Search box
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.entity_filter)
                            .hint_text("Filter…")
                            .desired_width(f32::INFINITY),
                    );
                });
                ui.separator();
                let filter_lower = self.entity_filter.to_lowercase();
                egui::ScrollArea::vertical()
                    .id_salt("entity_list")
                    .max_height(available.y - 32.0)                    .show(ui, |ui| {
                        let mut new_selected = self.selected;
                        let mut visible_count = 0usize;
                        for (i, entity) in self.entities.iter().enumerate() {
                            if !filter_lower.is_empty()
                                && !entity.name.to_lowercase().contains(&filter_lower)
                            {
                                continue;
                            }
                            visible_count += 1;
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
                        } else if visible_count == 0 {
                            ui.label(
                                egui::RichText::new("No matches.")
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

                // Viewport placeholder — right-click opens the pie menu.
                let (rect, response) = ui.allocate_exact_size(
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
                // Hint text (only when pie menu is closed so they don't overlap)
                if self.pie_menu_pos.is_none() {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🌐  3-D Viewport\nRendering integration pending\n\nRight-click for pie menu",
                        egui::FontId::proportional(13.0),
                        Color32::from_rgb(90, 90, 115),
                    );
                }

                // ── Pie menu ─────────────────────────────────────────────────
                // Open on right-click inside the viewport.
                if response.secondary_clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.pie_menu_pos = Some(pos);
                        self.pie_hovered = None;
                    }
                }

                if let Some(center) = self.pie_menu_pos {
                    // Update hover slice from current pointer position.
                    let pointer = ui.input(|i| i.pointer.latest_pos());
                    self.pie_hovered = pointer.and_then(|p| Self::pie_slice_at(center, p));

                    // Draw the pie menu on top of the viewport.
                    Self::draw_pie_menu(painter, center, self.pie_hovered, self.selected.is_some());

                    // Primary click: execute hovered action and close.
                    if response.clicked() {
                        if let Some(slice) = self.pie_hovered {
                            self.execute_pie_action(slice);
                        }
                        self.pie_menu_pos = None;
                        self.pie_hovered = None;
                    }

                    // Escape or click outside the viewport closes the menu.
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.pie_menu_pos = None;
                        self.pie_hovered = None;
                    }

                    // Request repaint so hover highlight stays responsive.
                    ui.ctx().request_repaint();
                }

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

// ---------------------------------------------------------------------------
// Pie menu colour helper
// ---------------------------------------------------------------------------

/// Returns the fill colour for pie slice `i` (0..PIE_N).
/// When `has_selection` is false, the Delete/Dup slices are dimmed.
fn pie_slice_color(i: usize, has_selection: bool) -> Color32 {
    match i {
        0 => Color32::from_rgb(45, 85, 165),  // Translate – blue
        1 => Color32::from_rgb(40, 140, 80),  // Rotate    – green
        2 => Color32::from_rgb(100, 55, 160), // Scale     – purple
        3 => Color32::from_rgb(35, 130, 145), // Add       – teal
        4 => {
            // Duplicate — dimmed when nothing selected
            if has_selection {
                Color32::from_rgb(160, 100, 35)
            } else {
                Color32::from_rgb(70, 60, 45)
            }
        }
        5 => {
            // Delete — dimmed when nothing selected
            if has_selection {
                Color32::from_rgb(165, 45, 45)
            } else {
                Color32::from_rgb(70, 42, 42)
            }
        }
        _ => Color32::from_rgb(60, 60, 70),
    }
}
