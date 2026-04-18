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

/// Asset kind filter state — `None` means "show all".
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum KindFilter {
    #[default]
    All,
    Only(AssetKind),
}

/// Asset Browser & Editor panel.
#[derive(Default)]
pub struct AssetEditor {
    filter: String,
    kind_filter: KindFilter,
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

        // Kind-filter buttons
        ui.horizontal(|ui| {
            let all_selected = self.kind_filter == KindFilter::All;
            if ui.selectable_label(all_selected, "All").clicked() {
                self.kind_filter = KindFilter::All;
                self.selected = None;
            }
            for (kind, label) in [
                (AssetKind::Texture, "🖼 Texture"),
                (AssetKind::Model, "📦 Model"),
                (AssetKind::Sound, "🔊 Sound"),
                (AssetKind::Scene, "🌐 Scene"),
                (AssetKind::Other, "📄 Other"),
            ] {
                let active = self.kind_filter == KindFilter::Only(kind);
                if ui.selectable_label(active, label).clicked() {
                    self.kind_filter = if active {
                        KindFilter::All
                    } else {
                        KindFilter::Only(kind)
                    };
                    self.selected = None;
                }
            }
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
                    // Text filter
                    if !filter_lower.is_empty()
                        && !entry.relative_path.to_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }
                    // Kind filter
                    if let KindFilter::Only(kind) = self.kind_filter {
                        if entry.kind != kind {
                            continue;
                        }
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

                // Resolve the absolute path so we can read metadata.
                let full_path = self
                    .scanned_root
                    .as_ref()
                    .map(|r| r.join(&entry.relative_path));

                egui::Grid::new("asset_detail")
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        ui.label("Path");
                        ui.label(&entry.relative_path);
                        ui.end_row();
                        ui.label("Type");
                        ui.label(format!("{} {:?}", entry.icon(), entry.kind));
                        ui.end_row();

                        if let Some(ref fp) = full_path {
                            if let Ok(meta) = std::fs::metadata(fp) {
                                ui.label("Size");
                                ui.label(human_file_size(meta.len()));
                                ui.end_row();
                                if let Ok(modified) = meta.modified() {
                                    ui.label("Modified");
                                    ui.label(format_system_time(modified));
                                    ui.end_row();
                                }
                            }
                        }
                    });
                ui.add_space(4.0);
                // Thumbnail placeholder
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 80.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 38));
                let thumb_text = match entry.kind {
                    novaforge_project::AssetKind::Texture => "🖼 Texture preview (pending)",
                    novaforge_project::AssetKind::Model => "📦 3-D model preview (pending)",
                    novaforge_project::AssetKind::Sound => "🔊 Audio waveform (pending)",
                    _ => "Thumbnail preview (pending)",
                };
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    thumb_text,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(90, 90, 110),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a byte count as a human-readable string (e.g. "12.3 KB").
fn human_file_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Format a [`std::time::SystemTime`] as a local-time string
/// (UTC ISO-8601 without sub-seconds, e.g. "2025-11-03 14:22:07 UTC").
fn format_system_time(t: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    let secs = match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return "—".to_string(),
    };

    // Manual decomposition from Unix timestamp (Gregorian calendar, UTC).
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;

    let days = secs / 86_400;
    let (year, month, day) = days_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02} {h:02}:{m:02}:{s:02} UTC")
}

/// Convert days since Unix epoch (1970-01-01) to (year, month, day).
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m, d)
}
