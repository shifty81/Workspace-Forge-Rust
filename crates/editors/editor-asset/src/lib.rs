//! Asset Browser & Editor panel for NovaForge Workspace.
//!
//! Reads the asset root from the loaded project context and lets the user
//! browse, filter, and inspect Nova-Forge asset files.
//!
//! Text-based assets (RON, TOML, Lua, GLSL, …) can be opened directly in the
//! Game File Editor via double-click, right-click context menu, or the
//! **"Open in File Editor"** button in the details pane.  The main app reads
//! [`AssetEditor::open_file_request`] each frame and routes it to the
//! appropriate tab.

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

/// Returns `true` for file extensions we can open in the Game File Editor.
fn is_text_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "toml"
            | "ron"
            | "json"
            | "yaml"
            | "yml"
            | "lua"
            | "txt"
            | "md"
            | "conf"
            | "cfg"
            | "ini"
            | "glsl"
            | "wgsl"
            | "vert"
            | "frag"
            | "comp"
            | "hlsl"
            | "py"
            | "sh"
            | "bat"
            | "rs"
    )
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
    /// Set by double-click or context menu; read and cleared by the main app
    /// each frame so it can open the file in the Game File Editor.
    pub open_file_request: Option<PathBuf>,
    /// Cached read-only text preview of the currently selected text asset.
    preview: Option<String>,
    /// The file path for which `preview` was last loaded — used to avoid
    /// re-reading on every frame.
    preview_path: Option<PathBuf>,
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
        self.preview = None;
        self.preview_path = None;
    }

    /// Build the platform-native absolute path for `relative_path`.
    fn full_path(&self, relative_path: &str) -> Option<PathBuf> {
        self.scanned_root
            .as_ref()
            .map(|r| r.join(relative_path.replace('/', std::path::MAIN_SEPARATOR_STR)))
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

        // Asset list  ──────────────────────────────────────────────────────────
        // Collect selection and open-request changes outside the borrow on
        // `self.assets` to avoid simultaneous mutable+immutable borrow errors.
        let mut new_selected = self.selected;
        let mut pending_open: Option<PathBuf> = None;

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 150.0)
            .show(ui, |ui| {
                let filter_lower = self.filter.to_lowercase();

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
                    let row_label = egui::RichText::new(format!(
                        "{} {}",
                        entry.icon(),
                        entry.relative_path
                    ));

                    // Build context-menu data before the response (avoids
                    // holding an immutable borrow on entry across closures).
                    let fp = self.full_path(&entry.relative_path);
                    let ext = std::path::Path::new(&entry.relative_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let is_text = is_text_ext(&ext);

                    let response = ui.selectable_label(selected, row_label);

                    if response.clicked() {
                        new_selected = Some(i);
                    }
                    // Double-click opens text assets immediately in the File Editor.
                    if response.double_clicked() {
                        new_selected = Some(i);
                        if is_text {
                            pending_open = fp.clone();
                        }
                    }

                    // Right-click context menu.
                    let menu_fp = fp.clone();
                    response.context_menu(|ui| {
                        if is_text
                            && ui
                                .button("📝 Open in File Editor")
                                .on_hover_text(
                                    "Open this file in the Game File Editor tab (or double-click)",
                                )
                                .clicked()
                        {
                            pending_open = menu_fp.clone();
                            ui.close_menu();
                        }
                        if ui
                            .button("📋 Copy Path")
                            .on_hover_text("Copy the absolute path to the clipboard")
                            .clicked()
                        {
                            if let Some(ref p) = menu_fp {
                                ui.ctx().copy_text(p.display().to_string());
                            }
                            ui.close_menu();
                        }
                    });
                }

                self.selected = new_selected;
            });

        // Commit any open-file request collected during the list loop.
        if pending_open.is_some() {
            self.open_file_request = pending_open;
        }

        // Details / inspector for selected asset  ─────────────────────────────
        if let Some(idx) = self.selected {
            // Clone entry so we don't hold a borrow on self.assets while we
            // mutate other fields (preview cache, open_file_request).
            if let Some(entry) = self.assets.get(idx).cloned() {
                ui.separator();
                ui.strong("Asset Details");

                let full_path = self.full_path(&entry.relative_path);
                let ext = std::path::Path::new(&entry.relative_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                let is_text = is_text_ext(ext);

                // Refresh text preview when the selected file changes.
                if is_text {
                    if self.preview_path.as_ref() != full_path.as_ref() {
                        self.preview = full_path.as_ref().and_then(|p| {
                            std::fs::read_to_string(p).ok().map(|s| {
                                s.lines().take(20).collect::<Vec<_>>().join("\n")
                            })
                        });
                        self.preview_path = full_path.clone();
                    }
                } else if self.preview_path.as_ref() != full_path.as_ref() {
                    self.preview = None;
                    self.preview_path = full_path.clone();
                }

                // Metadata grid.
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

                if is_text {
                    // "Open in File Editor" action button.
                    if ui
                        .button("📝 Open in File Editor")
                        .on_hover_text("Open in the Game File Editor tab  (or double-click the asset)")
                        .clicked()
                    {
                        if let Some(ref p) = full_path {
                            self.open_file_request = Some(p.clone());
                        }
                    }

                    // Read-only text preview.
                    if let Some(ref preview) = self.preview {
                        ui.separator();
                        ui.label(
                            egui::RichText::new("Preview (first 20 lines):")
                                .size(11.0)
                                .italics()
                                .color(Color32::from_rgb(140, 140, 165)),
                        );
                        let mut buf = preview.clone();
                        egui::ScrollArea::vertical()
                            .max_height(120.0)
                            .id_salt("asset_preview")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut buf)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .interactive(false),
                                );
                            });
                    }
                } else {
                    // Thumbnail placeholder for images, models, sounds, etc.
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 80.0),
                        egui::Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 38));
                    let thumb_text = match entry.kind {
                        AssetKind::Texture => "🖼 Texture preview (pending)",
                        AssetKind::Model => "📦 3-D model preview (pending)",
                        AssetKind::Sound => "🔊 Audio waveform (pending)",
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
