//! Asset Browser & Editor panel for NovaForge Workspace.
//!
//! Reads the asset root from the loaded project context and lets the user
//! browse, filter, and inspect Nova-Forge asset files.

use egui::Color32;
use novaforge_project::{scan_assets, AssetKind};
use novaforge_ui::{EditorPanel, PanelContext};
use std::path::PathBuf;

/// One row in the asset list — a file found on disk under the asset root.
#[derive(Clone)]
struct AssetEntry {
    /// Path relative to the asset root (forward-slash separated).
    relative_path: String,
    kind: AssetKind,
}

impl AssetEntry {
    fn icon(&self) -> &'static str {
        self.kind.icon()
    }
}

/// Asset Browser & Editor panel.
#[derive(Default)]
pub struct AssetEditor {
    filter: String,
    selected: Option<usize>,
    assets: Vec<AssetEntry>,
    /// The root we last scanned so we can detect when it changes.
    scanned_root: Option<PathBuf>,
}

impl AssetEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// (Re-)scan `root` and populate the asset list.
    fn scan(&mut self, root: PathBuf) {
        let raw = scan_assets(&root, 4);
        self.assets = raw
            .into_iter()
            .map(|e| AssetEntry {
                relative_path: e.relative_path,
                kind: e.kind,
            })
            .collect();
        self.selected = None;
        self.scanned_root = Some(root);
    }
}

impl EditorPanel for AssetEditor {
    fn title(&self) -> &str {
        "Asset Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        // Auto-scan when the asset root changes (e.g. a project was just opened).
        if let Some(root) = ctx.asset_root.as_ref() {
            let needs_scan = self.scanned_root.as_ref() != Some(root);
            if needs_scan {
                self.scan(root.clone());
            }
        }

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
                    if let Some(root) = ctx.asset_root.clone() {
                        self.scan(root);
                    }
                }
            });
        });

        if let Some(root) = ctx.asset_root.as_ref() {
            ui.label(
                egui::RichText::new(format!("Root: {}", root.display()))
                    .size(10.0)
                    .color(Color32::from_rgb(130, 130, 150)),
            );
        } else {
            ui.label(
                egui::RichText::new("Open a project to browse its assets.")
                    .italics()
                    .color(Color32::from_rgb(120, 120, 140)),
            );
        }

        ui.separator();

        // Asset list
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 120.0)
            .show(ui, |ui| {
                let filter_lower = self.filter.to_lowercase();
                let mut new_selected = self.selected;

                if self.assets.is_empty() && self.scanned_root.is_some() {
                    ui.label(
                        egui::RichText::new("No assets found in the project asset root.")
                            .italics()
                            .color(Color32::from_rgb(120, 120, 140)),
                    );
                }

                for (i, entry) in self.assets.iter().enumerate() {
                    if !filter_lower.is_empty()
                        && !entry.relative_path.to_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }

                    let selected = self.selected == Some(i);
                    let label =
                        egui::RichText::new(format!("{} {}", entry.icon(), entry.relative_path));
                    if ui.selectable_label(selected, label).clicked() {
                        new_selected = Some(i);
                    }
                }

                self.selected = new_selected;
            });

        // Preview / inspector for selected asset
        if let Some(idx) = self.selected {
            if let Some(entry) = self.assets.get(idx) {
                ui.separator();
                ui.strong("Asset Details");
                egui::Grid::new("asset_detail")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Path");
                        ui.label(&entry.relative_path);
                        ui.end_row();
                        ui.label("Type");
                        ui.label(entry.icon());
                        ui.end_row();
                    });
                ui.add_space(4.0);
                // Thumbnail placeholder
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 80.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 38));
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
