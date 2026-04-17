//! UI Layout Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// Kind of UI widget on the design canvas.
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
    #[allow(dead_code)]
    selected: bool,
}

/// UI Layout Editor panel.
///
/// Provides a drag-and-drop canvas for designing in-game UI layouts.
/// Full widget binding and property inspector will be added in a later phase.
pub struct UiEditorPanel {
    widgets: Vec<UiWidget>,
    #[allow(dead_code)]
    canvas_offset: egui::Vec2,
    dragging: Option<usize>,
    drag_start: egui::Pos2,
    widget_start: egui::Pos2,
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
                    selected: false,
                },
                UiWidget {
                    label: "Health Bar".to_string(),
                    kind: WidgetKind::Label,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(60.0, 60.0),
                        egui::vec2(120.0, 20.0),
                    ),
                    selected: false,
                },
                UiWidget {
                    label: "Minimap".to_string(),
                    kind: WidgetKind::Panel,
                    rect: egui::Rect::from_min_size(
                        egui::pos2(240.0, 30.0),
                        egui::vec2(80.0, 80.0),
                    ),
                    selected: false,
                },
            ],
            canvas_offset: egui::Vec2::ZERO,
            dragging: None,
            drag_start: egui::Pos2::ZERO,
            widget_start: egui::Pos2::ZERO,
        }
    }
}

impl EditorPanel for UiEditorPanel {
    fn title(&self) -> &str {
        "UI Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
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
                    selected: false,
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
                    selected: false,
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
                    selected: false,
                });
            }
            ui.separator();
            ui.label(format!("{} widgets", self.widgets.len()));
        });

        ui.separator();

        let available = ui.available_size();
        let (canvas_rect, response) =
            ui.allocate_exact_size(available, egui::Sense::click_and_drag());

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

        // Handle drag
        if response.drag_started() {
            self.dragging = None;
            if let Some(pos) = response.interact_pointer_pos() {
                for (i, w) in self.widgets.iter().enumerate() {
                    let screen_rect = w.rect.translate(canvas_rect.min.to_vec2());
                    if screen_rect.contains(pos) {
                        self.dragging = Some(i);
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

        // Draw widgets
        for (i, widget) in self.widgets.iter_mut().enumerate() {
            let screen_rect = widget.rect.translate(canvas_rect.min.to_vec2());
            if !canvas_rect.intersects(screen_rect) {
                continue;
            }
            let is_dragged = self.dragging == Some(i);
            let fill = if is_dragged {
                Color32::from_rgb(60, 90, 130)
            } else {
                widget.kind.fill_color()
            };
            painter.rect_filled(screen_rect, 4.0, fill);
            painter.rect_stroke(
                screen_rect,
                4.0,
                egui::Stroke::new(1.5, Color32::from_rgb(120, 140, 180)),
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
            "Drag widgets to reposition  •  full property binding pending",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(70, 70, 90),
        );
    }
}
