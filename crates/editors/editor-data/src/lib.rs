//! Data Table Editor panel for NovaForge Workspace.
//!
//! Provides a spreadsheet-style editor for game data tables such as item
//! definitions, NPC stats, loot tables, and configuration values.
//! Rows can be added, edited, deleted, and saved to TOML.

use novaforge_ui::{EditorPanel, PanelContext};
use serde::{Deserialize, Serialize};

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
        }
    }
}

impl DataEditor {
    /// Serialise all rows to TOML and write to `path`.
    fn save_to_toml(&mut self, ctx: &PanelContext) {
        // Derive save path from the asset root: <asset_root>/data/data_table.toml
        let save_path = ctx
            .asset_root
            .as_ref()
            .map(|r| r.join("data").join("data_table.toml"));

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
            if ui.button("🗑 Delete").clicked() {
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

        // Table
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 140.0)
            .show(ui, |ui| {
                egui::Grid::new("data_table")
                    .num_columns(5)
                    .striped(true)
                    .min_col_width(80.0)
                    .show(ui, |ui| {
                        // Header
                        ui.strong("ID");
                        ui.strong("Name");
                        ui.strong("Type");
                        ui.strong("Value");
                        ui.strong("Tags");
                        ui.end_row();

                        let mut new_selected = self.selected_row;

                        for (i, row) in self.rows.iter().enumerate() {
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
