//! Game File Editor panel for NovaForge Workspace.
//!
//! Provides an inline text editor for any text-based asset file
//! (RON, TOML, Lua, GLSL, JSON, …).  When the Workspace Browser
//! selects a supported file via [`PanelContext::selected_file`] the
//! editor opens it automatically.  The user can edit the text and
//! save back to disk with the **💾 Save** button.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Text-file detection
// ---------------------------------------------------------------------------

/// Returns `true` for extensions we treat as plain-text / editable.
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

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// Game File Editor panel.
///
/// Opens text-based Nova-Forge source / config files for inline editing.
#[derive(Default)]
pub struct GameFileEditor {
    /// The file currently open (absolute path).
    open_path: Option<PathBuf>,
    /// Text content buffer.
    content: String,
    /// `true` when the buffer has unsaved changes.
    dirty: bool,
    /// Status / feedback message shown below the toolbar.
    status: String,
    /// The last `selected_file` seen from [`PanelContext`] — used to
    /// detect when the Workspace Browser picks a new file.
    last_ctx_file: Option<PathBuf>,
}

impl GameFileEditor {
    /// Load the file at `path` into the editor buffer.
    fn open(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                self.content = text;
                self.dirty = false;
                self.status = format!("Opened: {}", path.display());
                self.open_path = Some(path);
            }
            Err(e) => {
                self.status = format!("Read error: {e}");
            }
        }
    }

    /// Write the buffer back to the open file.
    fn save(&mut self) {
        let Some(ref path) = self.open_path else {
            self.status = "No file open.".to_string();
            return;
        };
        match std::fs::write(path, &self.content) {
            Ok(()) => {
                self.dirty = false;
                self.status = format!("Saved → {}", path.display());
            }
            Err(e) => {
                self.status = format!("Write error: {e}");
            }
        }
    }

    /// Save the open file only if there are unsaved changes.
    /// Called by the main app for the Ctrl+S keyboard shortcut.
    pub fn save_if_dirty(&mut self) {
        if self.dirty {
            self.save();
        }
    }
}

impl EditorPanel for GameFileEditor {
    fn title(&self) -> &str {
        "File Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        // Auto-open when the Workspace Browser selects a new text file.
        if ctx.selected_file != self.last_ctx_file {
            self.last_ctx_file = ctx.selected_file.clone();
            if let Some(ref path) = ctx.selected_file.clone() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if is_text_ext(ext) {
                    self.open(path.clone());
                }
            }
        }

        // ── Toolbar ──────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if let Some(ref path) = self.open_path {
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                let badge = if self.dirty { "✏ " } else { "📄 " };
                let dirty_star = if self.dirty { " *" } else { "" };
                ui.strong(format!("{badge}{fname}{dirty_star}"));
                ui.separator();
                if ui
                    .add_enabled(self.dirty, egui::Button::new("💾 Save"))
                    .on_hover_text("Write buffer to disk")
                    .clicked()
                {
                    self.save();
                }
                if ui
                    .button("✖ Close")
                    .on_hover_text("Close without saving")
                    .clicked()
                {
                    self.open_path = None;
                    self.content.clear();
                    self.dirty = false;
                    self.status = String::new();
                }
            } else {
                ui.label(
                    egui::RichText::new("No file open.")
                        .italics()
                        .color(Color32::from_rgb(120, 120, 140)),
                );
                ui.label(
                    egui::RichText::new("Click a text file in the Workspace Browser.")
                        .size(11.0)
                        .color(Color32::from_rgb(100, 100, 120)),
                );
            }
        });

        // Status line
        if !self.status.is_empty() {
            ui.label(
                egui::RichText::new(&self.status)
                    .size(11.0)
                    .color(Color32::from_rgb(160, 200, 160)),
            );
        }

        // Full path hint
        if let Some(ref path) = self.open_path {
            ui.label(
                egui::RichText::new(path.display().to_string())
                    .size(10.0)
                    .color(Color32::from_rgb(100, 100, 120)),
            );
        }

        ui.separator();

        // ── Editor area ───────────────────────────────────────────────────────
        if self.open_path.is_some() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(50),
                );
                if response.changed() {
                    self.dirty = true;
                }
            });
        } else {
            // Empty-state placeholder
            let avail = ui.available_rect_before_wrap();
            let painter = ui.painter_at(avail);
            painter.rect_filled(avail, 6.0, Color32::from_rgb(20, 20, 26));
            painter.rect_stroke(
                avail,
                6.0,
                egui::Stroke::new(1.0, Color32::from_rgb(45, 45, 58)),
                egui::StrokeKind::Middle,
            );
            painter.text(
                avail.center() - egui::vec2(0.0, 12.0),
                egui::Align2::CENTER_CENTER,
                "📝",
                egui::FontId::proportional(36.0),
                Color32::from_rgb(70, 70, 90),
            );
            painter.text(
                avail.center() + egui::vec2(0.0, 24.0),
                egui::Align2::CENTER_CENTER,
                "Game File Editor\nClick a text file in the Workspace Browser to open it",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(80, 80, 105),
            );
        }
    }
}
