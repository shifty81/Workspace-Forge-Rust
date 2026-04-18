//! Build Tool panel for NovaForge Workspace.
//!
//! Provides Build / Clean / Run buttons that call `nova-forge.sh` and stream
//! the output live into a scrollable log pane.

use egui::Color32;
use novaforge_build::{BuildCommand, BuildRunner};
use novaforge_ui::{EditorPanel, PanelContext};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// Build Tool panel.
pub struct BuildToolPanel {
    log_lines: Vec<(LogLevel, String)>,
    receiver: Option<Receiver<String>>,
    runner: Option<BuildRunner>,
    running: bool,
    auto_scroll: bool,
}

#[derive(Clone, Copy)]
enum LogLevel {
    Info,
    Done,
    Error,
}

impl LogLevel {
    fn colour(self) -> Color32 {
        match self {
            LogLevel::Info => Color32::from_rgb(200, 200, 215),
            LogLevel::Done => Color32::from_rgb(100, 220, 100),
            LogLevel::Error => Color32::from_rgb(240, 100, 100),
        }
    }

    fn from_line(line: &str) -> Self {
        if line.starts_with("[done]") {
            LogLevel::Done
        } else if line.starts_with("[error]") {
            LogLevel::Error
        } else {
            LogLevel::Info
        }
    }
}

impl Default for BuildToolPanel {
    fn default() -> Self {
        Self {
            log_lines: vec![(
                LogLevel::Info,
                "NovaForge Build Tool ready. Open a project and press Build.".to_string(),
            )],
            receiver: None,
            runner: None,
            running: false,
            auto_scroll: true,
        }
    }
}

impl BuildToolPanel {
    /// Public entry point so the menu bar (or keyboard shortcuts) can trigger a
    /// build without the user having to switch to the Build panel first.
    pub fn trigger(&mut self, cmd: BuildCommand, nova_forge_path: Option<&PathBuf>) {
        self.spawn_command(cmd, nova_forge_path);
    }

    fn spawn_command(&mut self, cmd: BuildCommand, nova_forge_path: Option<&PathBuf>) {
        match nova_forge_path {
            Some(path) => {
                let runner = BuildRunner::new(path.clone());
                let rx = runner.spawn(cmd);
                self.runner = Some(runner);
                self.receiver = Some(rx);
                self.running = true;
                self.log_lines
                    .push((LogLevel::Info, format!("--- {} ---", cmd.label())));
            }
            None => {
                self.log_lines.push((
                    LogLevel::Error,
                    "[error] No project loaded — cannot resolve nova-forge.sh path.".to_string(),
                ));
            }
        }
    }

    fn poll_log(&mut self) {
        if let Some(rx) = &self.receiver {
            loop {
                match rx.try_recv() {
                    Ok(line) => {
                        let level = LogLevel::from_line(&line);
                        if matches!(level, LogLevel::Done | LogLevel::Error) {
                            self.running = false;
                        }
                        self.log_lines.push((level, line));
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.running = false;
                        self.receiver = None;
                        break;
                    }
                }
            }
        }
    }
}

impl EditorPanel for BuildToolPanel {
    fn title(&self) -> &str {
        "Build Tool"
    }

    fn background_update(&mut self) {
        self.poll_log();
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        self.poll_log();

        let nova_forge_path = ctx.nova_forge_path.as_ref();

        // Toolbar
        ui.horizontal(|ui| {
            let busy = self.running;
            ui.add_enabled_ui(!busy, |ui| {
                if ui
                    .button("🔨 Build")
                    .on_hover_text("cargo build (debug)")
                    .clicked()
                {
                    self.spawn_command(BuildCommand::Build, nova_forge_path);
                }
                if ui
                    .button("🚀 Release")
                    .on_hover_text("cargo build --release")
                    .clicked()
                {
                    self.spawn_command(BuildCommand::Release, nova_forge_path);
                }
                if ui.button("🧹 Clean").on_hover_text("cargo clean").clicked() {
                    self.spawn_command(BuildCommand::Clean, nova_forge_path);
                }
                if ui
                    .button("▶ Run")
                    .on_hover_text("Build and run nova-forge client")
                    .clicked()
                {
                    self.spawn_command(BuildCommand::Run, nova_forge_path);
                }
                if ui
                    .button("🖥 Server")
                    .on_hover_text("Build and run dedicated server")
                    .clicked()
                {
                    self.spawn_command(BuildCommand::RunServer, nova_forge_path);
                }
                if ui.button("🧪 Test").on_hover_text("cargo test").clicked() {
                    self.spawn_command(BuildCommand::Test, nova_forge_path);
                }
            });

            ui.separator();

            if busy {
                ui.spinner();
                ui.label("Building…");
            } else {
                ui.label("Ready");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Clear").clicked() {
                    self.log_lines.clear();
                }
                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
            });
        });

        ui.separator();

        // Log output
        let text_style = egui::TextStyle::Monospace;
        let row_height = ui.text_style_height(&text_style);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(self.auto_scroll)
            .show_rows(ui, row_height, self.log_lines.len(), |ui, range| {
                for i in range {
                    if let Some((level, line)) = self.log_lines.get(i) {
                        ui.label(egui::RichText::new(line).monospace().color(level.colour()));
                    }
                }
            });

        // Keep repainting while the build is running so the log streams live.
        if self.running {
            ui.ctx().request_repaint();
        }
    }
}
