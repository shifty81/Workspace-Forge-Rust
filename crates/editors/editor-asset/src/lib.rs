//! Asset Browser & Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// One row in the asset list.
#[derive(Clone)]
struct AssetEntry {
    name: String,
    kind: AssetKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssetKind {
    Texture,
    Model,
    Sound,
    Scene,
    Other,
}

impl AssetKind {
    fn icon(self) -> &'static str {
        match self {
            Self::Texture => "🖼",
            Self::Model => "📦",
            Self::Sound => "🔊",
            Self::Scene => "🌐",
            Self::Other => "📄",
        }
    }
}

/// Asset Browser & Editor panel.
#[derive(Default)]
pub struct AssetEditor {
    filter: String,
    selected: Option<usize>,
    assets: Vec<AssetEntry>,
}

impl AssetEditor {
    pub fn new() -> Self {
        // Seed with some representative placeholder entries.
        Self {
            assets: vec![
                AssetEntry { name: "terrain.png".to_string(), kind: AssetKind::Texture },
                AssetEntry { name: "player.vox".to_string(), kind: AssetKind::Model },
                AssetEntry { name: "ambient.ogg".to_string(), kind: AssetKind::Sound },
                AssetEntry { name: "world/main.ron".to_string(), kind: AssetKind::Scene },
                AssetEntry { name: "item/sword.ron".to_string(), kind: AssetKind::Other },
            ],
            ..Default::default()
        }
    }
}

impl EditorPanel for AssetEditor {
    fn title(&self) -> &str {
        "Asset Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter assets…")
                    .desired_width(180.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⟳ Refresh").clicked() {
                    // TODO: re-scan asset_root
                }
            });
        });

        if let Some(root) = ctx.asset_root.as_ref() {
            ui.label(
                egui::RichText::new(format!("Root: {}", root.display()))
                    .size(10.0)
                    .color(Color32::from_rgb(130, 130, 150)),
            );
        }

        ui.separator();

        // Asset list
        egui::ScrollArea::vertical().show(ui, |ui| {
            let filter_lower = self.filter.to_lowercase();
            let mut new_selected = self.selected;

            for (i, entry) in self.assets.iter().enumerate() {
                if !filter_lower.is_empty()
                    && !entry.name.to_lowercase().contains(&filter_lower)
                {
                    continue;
                }

                let selected = self.selected == Some(i);
                let label = egui::RichText::new(format!("{} {}", entry.kind.icon(), entry.name));
                if ui.selectable_label(selected, label).clicked() {
                    new_selected = Some(i);
                }
            }

            self.selected = new_selected;
        });

        // Preview / inspector
        if let Some(idx) = self.selected {
            if let Some(entry) = self.assets.get(idx) {
                ui.separator();
                ui.strong("Asset Details");
                egui::Grid::new("asset_detail").num_columns(2).show(ui, |ui| {
                    ui.label("Name");
                    ui.label(&entry.name);
                    ui.end_row();
                    ui.label("Type");
                    ui.label(entry.kind.icon());
                    ui.end_row();
                });
                ui.add_space(4.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 80.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 38));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Thumbnail preview (pending)",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(90, 90, 110),
                );
            }
        }
    }
}
