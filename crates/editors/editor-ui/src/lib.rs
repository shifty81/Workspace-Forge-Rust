//! UI Layout Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Height reserved for the property inspector when a widget is selected.
const INSPECTOR_HEIGHT: f32 = 110.0;
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum WidgetKind {
    #[default]
    Panel,
    Label,
    Button,
}

impl WidgetKind {
    fn label(self) -> &'static str {
        match self {
            WidgetKind::Panel => "Panel",
            WidgetKind::Label => "Label",
            WidgetKind::Button => "Button",
        }
    }

    fn default_size(self) -> egui::Vec2 {
        match self {
            WidgetKind::Panel => egui::vec2(120.0, 60.0),
            WidgetKind::Label => egui::vec2(100.0, 20.0),
            WidgetKind::Button => egui::vec2(90.0, 28.0),
        }
    }

    fn fill_color(self) -> egui::Color32 {
        match self {
            WidgetKind::Panel => egui::Color32::from_rgb(45, 55, 75),
            WidgetKind::Label => egui::Color32::from_rgb(40, 60, 50),
            WidgetKind::Button => egui::Color32::from_rgb(60, 50, 80),
        }
    }
}

/// A widget placed on the design canvas.
#[derive(Clone)]
struct UiWidget {
    label: String,
    kind: WidgetKind,
    rect: egui::Rect,
}

/// UI Layout Editor panel.
///
/// Provides a drag-and-drop canvas for designing in-game UI layouts.
/// When a widget is selected an inspector panel appears below the canvas
/// showing its label, type, position, and size.
///
/// The layout can be saved to and loaded from `<asset_root>/ui/ui_layout.toml`.
pub struct UiEditorPanel {
    widgets: Vec<UiWidget>,
    #[allow(dead_code)]
    canvas_offset: egui::Vec2,
    /// Index of the widget currently being dragged.
    dragging: Option<usize>,
    drag_start: egui::Pos2,
    widget_start: egui::Pos2,
    /// Index of the selected widget (click to select, Delete button to remove).
    selected_widget: Option<usize>,
    /// Status message shown in the toolbar (save/load feedback).
    save_status: String,
}

impl Default for UiEditorPanel {
    fn default() -> Self {
        Self {
            widgets: vec![
                UiWidget {
                    label: "HUD Panel".to_string(),
                    kind: WidgetKind::Panel,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(40.0, 40.0),
                        egui::vec2(160.0, 80.0),
                    ),
                },
                UiWidget {
                    label: "Health Bar".to_string(),
                    kind: WidgetKind::Label,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(60.0, 60.0),
                        egui::vec2(120.0, 20.0),
                    ),
                },
                UiWidget {
                    label: "Minimap".to_string(),
                    kind: WidgetKind::Panel,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(240.0, 30.0),
                        egui::vec2(80.0, 80.0),
                    ),
                },
            ],
            canvas_offset: egui::Vec2::ZERO,
            dragging: None,
            drag_start: egui::Pos2::ZERO,
            widget_start: egui::Pos2::ZERO,
            selected_widget: None,
            save_status: String::new(),
        }
    }
}

impl EditorPanel for UiEditorPanel {
    fn title(&self) -> &str {
        "UI Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        ui.horizontal(|ui| {
            if ui.button("＋ Panel").clicked() {
                let kind = WidgetKind::Panel;
                self.widgets.push(UiWidget {
                    label: format!("Panel {}", self.widgets.len() + 1),
                    kind,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(20.0 + self.widgets.len() as f32 * 10.0, 20.0),
                        kind.default_size(),
                    ),
                });
            }
            if ui.button("＋ Label").clicked() {
                let kind = WidgetKind::Label;
                self.widgets.push(UiWidget {
                    label: format!("Label {}", self.widgets.len() + 1),
                    kind,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(20.0 + self.widgets.len() as f32 * 10.0, 60.0),
                        kind.default_size(),
                    ),
                });
            }
            if ui.button("＋ Button").clicked() {
                let kind = WidgetKind::Button;
                self.widgets.push(UiWidget {
                    label: format!("Button {}", self.widgets.len() + 1),
                    kind,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(20.0 + self.widgets.len() as f32 * 10.0, 100.0),
                        kind.default_size(),
                    ),
                });
            }
            ui.separator();
            let delete_enabled = self.selected_widget.is_some();
            if ui
                .add_enabled(delete_enabled, egui::Button::new("🗑 Delete"))
                .on_hover_text("Delete selected widget")
                .clicked()
            {
                if let Some(idx) = self.selected_widget {
                    if idx < self.widgets.len() {
                        self.widgets.remove(idx);
                    }
                    self.selected_widget = None;
                    self.dragging = None;
                }
            }
            ui.separator();
            if ui
                .button("💾 Save")
                .on_hover_text("Save layout to <asset_root>/ui/ui_layout.toml")
                .clicked()
            {
                self.save_layout(ctx);
            }
            if ui
                .button("📂 Load")
                .on_hover_text("Load layout from <asset_root>/ui/ui_layout.toml")
                .clicked()
            {
                self.load_layout(ctx);
            }
            ui.separator();
            ui.label(format!("{} widgets", self.widgets.len()));
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

        // Reserve space for the property inspector if a widget is selected.
        let inspector_shown = self.selected_widget.is_some();
        let canvas_height = {
            let avail = ui.available_size();
            if inspector_shown {
                (avail.y - INSPECTOR_HEIGHT - 4.0).max(40.0)
            } else {
                avail.y
            }
        };
        let canvas_width = ui.available_width();

        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(canvas_width, canvas_height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(canvas_rect);
        // Checkerboard background
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(28, 28, 34));
        let tile = 16.0;
        let cols = (canvas_rect.width() / tile) as usize + 1;
        let rows = (canvas_rect.height() / tile) as usize + 1;
        for row in 0..rows {
            for col in 0..cols {
                if (row + col) % 2 == 0 {
                    let rect = egui::Rect::from_min_size(
                        canvas_rect.min + egui::vec2(col as f32 * tile, row as f32 * tile),
                        egui::vec2(tile, tile),
                    );
                    painter.rect_filled(rect, 0.0, Color32::from_rgb(34, 34, 42));
                }
            }
        }

        // Handle drag and click-to-select
        if response.drag_started() {
            self.dragging = None;
            if let Some(pos) = response.interact_pointer_pos() {
                for (i, w) in self.widgets.iter().enumerate() {
                    let screen_rect = w.rect.translate(canvas_rect.min.to_vec2());
                    if screen_rect.contains(pos) {
                        self.dragging = Some(i);
                        self.selected_widget = Some(i);
                        self.drag_start = pos;
                        self.widget_start = w.rect.min;
                        break;
                    }
                }
            }
        }
        if let Some(idx) = self.dragging {
            if let Some(pos) = response.interact_pointer_pos() {
                let delta = pos - self.drag_start;
                if let Some(w) = self.widgets.get_mut(idx) {
                    let new_min = self.widget_start + delta;
                    w.rect = egui::Rect::from_min_size(new_min, w.rect.size());
                }
            }
        }
        if response.drag_stopped() {
            self.dragging = None;
        }
        // Click on canvas background (not a drag) deselects.
        if response.clicked() {
            let hit = response.interact_pointer_pos().map(|pos| {
                self.widgets
                    .iter()
                    .any(|w| w.rect.translate(canvas_rect.min.to_vec2()).contains(pos))
            });
            if hit != Some(true) {
                self.selected_widget = None;
            }
        }

        // Draw widgets
        let selected_widget = self.selected_widget;
        let dragging = self.dragging;
        for (i, widget) in self.widgets.iter().enumerate() {
            let screen_rect = widget.rect.translate(canvas_rect.min.to_vec2());
            if !canvas_rect.intersects(screen_rect) {
                continue;
            }
            let is_dragged = dragging == Some(i);
            let is_selected = selected_widget == Some(i);
            let fill = if is_dragged {
                Color32::from_rgb(60, 90, 130)
            } else {
                widget.kind.fill_color()
            };
            painter.rect_filled(screen_rect, 4.0, fill);
            let (stroke_width, stroke_color) = if is_selected {
                (2.5, Color32::from_rgb(220, 220, 255))
            } else {
                (1.5, Color32::from_rgb(120, 140, 180))
            };
            painter.rect_stroke(
                screen_rect,
                4.0,
                egui::Stroke::new(stroke_width, stroke_color),
                egui::StrokeKind::Middle,
            );
            let display = format!("[{}] {}", widget.kind.label(), widget.label);
            painter.text(
                screen_rect.center(),
                egui::Align2::CENTER_CENTER,
                &display,
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        painter.text(
            canvas_rect.left_bottom() + egui::vec2(8.0, -10.0),
            egui::Align2::LEFT_BOTTOM,
            "Click to select  •  drag to reposition  •  🗑 Delete removes selected widget",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(70, 70, 90),
        );

        // ── Property Inspector ────────────────────────────────────────────────
        if let Some(idx) = self.selected_widget {
            if let Some(widget) = self.widgets.get_mut(idx) {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.strong("Inspector");
                    ui.label(
                        egui::RichText::new(format!(
                            "— {} ({})",
                            widget.label,
                            widget.kind.label()
                        ))
                        .size(11.0)
                        .color(Color32::from_rgb(160, 180, 210)),
                    );
                });
                egui::Grid::new("ui_inspector")
                    .num_columns(4)
                    .spacing([6.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Label");
                        ui.add(egui::TextEdit::singleline(&mut widget.label).desired_width(140.0));
                        ui.label("Type");
                        ui.label(widget.kind.label());
                        ui.end_row();

                        ui.label("Position");
                        ui.add(
                            egui::DragValue::new(&mut widget.rect.min.x)
                                .prefix("X ")
                                .speed(1.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut widget.rect.min.y)
                                .prefix("Y ")
                                .speed(1.0),
                        );
                        // Keep max consistent with min after dragging.
                        widget.rect =
                            egui::Rect::from_min_size(widget.rect.min, widget.rect.size());
                        ui.end_row();

                        ui.label("Size");
                        let mut w = widget.rect.width();
                        let mut h = widget.rect.height();
                        ui.add(
                            egui::DragValue::new(&mut w)
                                .prefix("W ")
                                .speed(1.0)
                                .range(4.0..=2000.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut h)
                                .prefix("H ")
                                .speed(1.0)
                                .range(4.0..=2000.0),
                        );
                        widget.rect = egui::Rect::from_min_size(widget.rect.min, egui::vec2(w, h));
                        ui.end_row();
                    });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Save / load helpers
// ---------------------------------------------------------------------------

/// Serialisable kind tag.
#[derive(Serialize, Deserialize, Clone, Copy)]
enum WidgetKindFile {
    Panel,
    Label,
    Button,
}

/// Serialisable form of a [`UiWidget`].
#[derive(Serialize, Deserialize)]
struct UiWidgetFile {
    label: String,
    kind: WidgetKindFile,
    /// [min_x, min_y, width, height]
    rect: [f32; 4],
}

/// Root structure for `ui_layout.toml`.
#[derive(Serialize, Deserialize)]
struct UiLayoutFile {
    widgets: Vec<UiWidgetFile>,
}

impl UiEditorPanel {
    fn layout_path(ctx: &PanelContext) -> Option<PathBuf> {
        ctx.asset_root
            .as_ref()
            .map(|r| r.join("ui").join("ui_layout.toml"))
    }

    fn save_layout(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::layout_path(ctx) else {
            self.save_status = "No project loaded — cannot save layout.".to_string();
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.save_status = format!("Directory error: {e}");
                return;
            }
        }
        let file = UiLayoutFile {
            widgets: self
                .widgets
                .iter()
                .map(|w| UiWidgetFile {
                    label: w.label.clone(),
                    kind: match w.kind {
                        WidgetKind::Panel => WidgetKindFile::Panel,
                        WidgetKind::Label => WidgetKindFile::Label,
                        WidgetKind::Button => WidgetKindFile::Button,
                    },
                    rect: [w.rect.min.x, w.rect.min.y, w.rect.width(), w.rect.height()],
                })
                .collect(),
        };
        match toml::to_string_pretty(&file) {
            Ok(content) => match std::fs::write(&path, content) {
                Ok(()) => self.save_status = format!("Saved → {}", path.display()),
                Err(e) => self.save_status = format!("Write error: {e}"),
            },
            Err(e) => self.save_status = format!("Serialise error: {e}"),
        }
    }

    fn load_layout(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::layout_path(ctx) else {
            self.save_status = "No project loaded — cannot load layout.".to_string();
            return;
        };
        if !path.exists() {
            self.save_status = format!("File not found: {}", path.display());
            return;
        }
        match std::fs::read_to_string(&path) {
            Err(e) => self.save_status = format!("Read error: {e}"),
            Ok(content) => match toml::from_str::<UiLayoutFile>(&content) {
                Err(e) => self.save_status = format!("Parse error: {e}"),
                Ok(file) => {
                    self.widgets = file
                        .widgets
                        .into_iter()
                        .map(|w| {
                            let kind = match w.kind {
                                WidgetKindFile::Panel => WidgetKind::Panel,
                                WidgetKindFile::Label => WidgetKind::Label,
                                WidgetKindFile::Button => WidgetKind::Button,
                            };
                            UiWidget {
                                label: w.label,
                                kind,
                                rect: egui::Rect::from_min_size(
                                    egui::pos2(w.rect[0], w.rect[1]),
                                    egui::vec2(w.rect[2], w.rect[3]),
                                ),
                            }
                        })
                        .collect();
                    self.selected_widget = None;
                    self.dragging = None;
                    let count = self.widgets.len();
                    self.save_status = format!("Loaded {count} widgets ← {}", path.display());
                }
            },
        }
    }
}
