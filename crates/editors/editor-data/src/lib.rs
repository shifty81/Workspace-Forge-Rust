//! Data Table Editor panel for NovaForge Workspace.
//!
//! Provides a spreadsheet-style editor for game data tables such as item
//! definitions, NPC stats, loot tables, and configuration values.
//! Rows can be added, edited, deleted, and saved to TOML.

use novaforge_ui::{EditorPanel, PanelContext};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single row in the data table.
#[derive(Clone, Serialize, Deserialize)]
struct DataRow {
    id: String,
    name: String,
    kind: String,
    value: String,
    tags: String,
}

/// Data Table Editor panel.
pub struct DataEditor {
    rows: Vec<DataRow>,
    selected_row: Option<usize>,
    filter: String,
    edit_buf: DataRow,
    /// Status message shown below the toolbar.
    save_status: String,
    /// Path from which data was last loaded / saved.
    last_path: Option<PathBuf>,
    /// Column index currently used for sorting (0 = ID … 4 = Tags), or `None`.
    sort_col: Option<usize>,
    /// `true` = ascending, `false` = descending.
    sort_asc: bool,
}

impl Default for DataEditor {
    fn default() -> Self {
        let placeholder = DataRow {
            id: String::new(),
            name: String::new(),
            kind: String::new(),
            value: String::new(),
            tags: String::new(),
        };
        Self {
            rows: vec![
                DataRow {
                    id: "item.sword.iron".to_string(),
                    name: "Iron Sword".to_string(),
                    kind: "Weapon".to_string(),
                    value: "150".to_string(),
                    tags: "melee,one-hand".to_string(),
                },
                DataRow {
                    id: "item.potion.health_s".to_string(),
                    name: "Small Health Potion".to_string(),
                    kind: "Consumable".to_string(),
                    value: "30".to_string(),
                    tags: "healing,quick".to_string(),
                },
                DataRow {
                    id: "npc.goblin".to_string(),
                    name: "Goblin".to_string(),
                    kind: "Enemy".to_string(),
                    value: "10".to_string(),
                    tags: "melee,hostile".to_string(),
                },
                DataRow {
                    id: "zone.startzone".to_string(),
                    name: "Starting Zone".to_string(),
                    kind: "Zone".to_string(),
                    value: "1".to_string(),
                    tags: "tutorial,safe".to_string(),
                },
            ],
            selected_row: None,
            filter: String::new(),
            edit_buf: placeholder,
            save_status: String::new(),
            last_path: None,
            sort_col: None,
            sort_asc: true,
        }
    }
}

impl DataEditor {
    /// Derive the canonical save/load path from the project context.
    fn toml_path(ctx: &PanelContext) -> Option<PathBuf> {
        ctx.asset_root
            .as_ref()
            .map(|r| r.join("data").join("data_table.toml"))
    }

    /// Load rows from `<asset_root>/data/data_table.toml`, replacing any
    /// existing rows.  Reports errors via `save_status`.
    fn load_from_toml(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::toml_path(ctx) else {
            self.save_status = "No project loaded — cannot determine load path.".to_string();
            return;
        };

        match std::fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.save_status = format!("File not found: {}", path.display());
            }
            Err(e) => {
                self.save_status = format!("Read error: {e}");
            }
            Ok(content) => {
                #[derive(Deserialize)]
                struct TableFile {
                    rows: Vec<DataRow>,
                }
                match toml::from_str::<TableFile>(&content) {
                    Ok(table) => {
                        self.rows = table.rows;
                        self.selected_row = None;
                        self.last_path = Some(path.clone());
                        self.save_status = format!("Loaded {} rows ← {}", self.rows.len(), path.display());
                    }
                    Err(e) => {
                        self.save_status = format!("Parse error: {e}");
                    }
                }
            }
        }
    }

    /// Serialise all rows to TOML and write to `path`.
    fn save_to_toml(&mut self, ctx: &PanelContext) {
        // Derive save path from the asset root: <asset_root>/data/data_table.toml
        let save_path = Self::toml_path(ctx);

        let Some(path) = save_path else {
            self.save_status = "No project loaded — cannot determine save path.".to_string();
            return;
        };

        // Ensure the parent directory exists.
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.save_status = format!("Directory error: {e}");
                return;
            }
        }

        // Serialise as TOML array of tables.
        #[derive(Serialize)]
        struct TableFile<'a> {
            rows: &'a [DataRow],
        }
        match toml::to_string_pretty(&TableFile { rows: &self.rows }) {
            Ok(content) => match std::fs::write(&path, content) {
                Ok(()) => {
                    self.last_path = Some(path.clone());
                    self.save_status = format!("Saved → {}", path.display());
                }
                Err(e) => {
                    self.save_status = format!("Write error: {e}");
                }
            },
            Err(e) => {
                self.save_status = format!("Serialise error: {e}");
            }
        }
    }
}

impl EditorPanel for DataEditor {
    fn title(&self) -> &str {
        "Data Editor"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter rows…")
                    .desired_width(180.0),
            );
            ui.separator();
            if ui.button("＋ Row").clicked() {
                self.rows.push(DataRow {
                    id: format!("new.entry.{}", self.rows.len()),
                    name: "New Entry".to_string(),
                    kind: "Item".to_string(),
                    value: "0".to_string(),
                    tags: String::new(),
                });
            }
            let delete_enabled = self.selected_row.is_some();
            if ui
                .add_enabled(delete_enabled, egui::Button::new("🗑 Delete"))
                .on_hover_text("Delete selected row")
                .clicked()
            {
                if let Some(idx) = self.selected_row {
                    if idx < self.rows.len() {
                        self.rows.remove(idx);
                        self.selected_row = None;
                    }
                }
            }
            if ui.button("💾 Save").clicked() {
                self.save_to_toml(ctx);
            }
            if ui.button("📂 Load").clicked() {
                self.load_from_toml(ctx);
            }
        });

        if !self.save_status.is_empty() {
            ui.label(
                egui::RichText::new(&self.save_status)
                    .size(11.0)
                    .color(egui::Color32::from_rgb(160, 200, 160)),
            );
        }

        ui.separator();

        let filter_lower = self.filter.to_lowercase();

        // Build a display-order index respecting the active sort.
        let mut display_indices: Vec<usize> = (0..self.rows.len()).collect();
        if let Some(col) = self.sort_col {
            let asc = self.sort_asc;
            display_indices.sort_by(|&a, &b| {
                let ra = &self.rows[a];
                let rb = &self.rows[b];
                let ord = match col {
                    0 => ra.id.cmp(&rb.id),
                    1 => ra.name.cmp(&rb.name),
                    2 => ra.kind.cmp(&rb.kind),
                    3 => ra.value.cmp(&rb.value),
                    4 => ra.tags.cmp(&rb.tags),
                    _ => std::cmp::Ordering::Equal,
                };
                if asc { ord } else { ord.reverse() }
            });
        }

        // Helper: build the column header label with a sort indicator.
        let col_header = |name: &str, idx: usize, sort_col: Option<usize>, sort_asc: bool| -> String {
            if sort_col == Some(idx) {
                format!("{name} {}", if sort_asc { "▲" } else { "▼" })
            } else {
                name.to_string()
            }
        };

        // Table
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 140.0)
            .show(ui, |ui| {
                egui::Grid::new("data_table")
                    .num_columns(5)
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        // Clickable column headers
                        for (ci, name) in ["ID", "Name", "Type", "Value", "Tags"].iter().enumerate() {
                            let label = col_header(name, ci, self.sort_col, self.sort_asc);
                            if ui
                                .add(egui::Button::new(egui::RichText::new(label).strong()).frame(false))
                                .on_hover_text("Click to sort")
                                .clicked()
                            {
                                if self.sort_col == Some(ci) {
                                    self.sort_asc = !self.sort_asc;
                                } else {
                                    self.sort_col = Some(ci);
                                    self.sort_asc = true;
                                }
                                self.selected_row = None;
                            }
                        }
                        ui.end_row();

                        let mut new_selected = self.selected_row;

                        for &i in &display_indices {
                            let row = &self.rows[i];
                            if !filter_lower.is_empty() {
                                let haystack = format!(
                                    "{} {} {} {} {}",
                                    row.id, row.name, row.kind, row.value, row.tags
                                )
                                .to_lowercase();
                                if !haystack.contains(&filter_lower) {
                                    continue;
                                }
                            }

                            let selected = self.selected_row == Some(i);

                            if ui.selectable_label(selected, &row.id).clicked() {
                                new_selected = Some(i);
                                self.edit_buf = row.clone();
                            }
                            ui.label(&row.name);
                            ui.label(&row.kind);
                            ui.label(&row.value);
                            ui.label(&row.tags);
                            ui.end_row();
                        }

                        self.selected_row = new_selected;
                    });
            });

        // Inline editor for selected row
        if let Some(idx) = self.selected_row {
            ui.separator();
            ui.strong("Edit Row");

            egui::Grid::new("row_edit").num_columns(2).show(ui, |ui| {
                ui.label("ID");
                ui.add(egui::TextEdit::singleline(&mut self.edit_buf.id).desired_width(240.0));
                ui.end_row();
                ui.label("Name");
                ui.text_edit_singleline(&mut self.edit_buf.name);
                ui.end_row();
                ui.label("Type");
                ui.text_edit_singleline(&mut self.edit_buf.kind);
                ui.end_row();
                ui.label("Value");
                ui.text_edit_singleline(&mut self.edit_buf.value);
                ui.end_row();
                ui.label("Tags");
                ui.text_edit_singleline(&mut self.edit_buf.tags);
                ui.end_row();
            });

            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    if let Some(row) = self.rows.get_mut(idx) {
                        *row = self.edit_buf.clone();
                    }
                }
                if ui.button("Revert").clicked() {
                    if let Some(row) = self.rows.get(idx) {
                        self.edit_buf = row.clone();
                    }
                }
            });
        }
    }
}
