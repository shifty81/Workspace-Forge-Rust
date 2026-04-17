//! NovaForge Workspace — launcher.
//!
//! Opens a small launcher window that lets the player:
//! - **Play** the Nova-Forge game client directly.
//! - **Host a LAN game** (starts a server and the client).
//! - **Open the Workspace** (opens the full `novaforge-editors` suite).
//!
//! A project file (`novaforge.workspace.toml`) can be opened to tell the
//! launcher where the Nova-Forge binary lives.

use eframe::egui;
use novaforge_project::{WorkspaceManifest, MANIFEST_FILE};
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 420.0])
            .with_resizable(false)
            .with_title("NovaForge Workspace"),
        ..Default::default()
    };

    eframe::run_native(
        "NovaForge Workspace",
        options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::default()))),
    )
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

#[derive(Default)]
struct LauncherApp {
    project: Option<WorkspaceManifest>,
    project_path_input: String,
    status: String,
    recent_projects: Vec<PathBuf>,
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_header(ui);
            ui.add_space(16.0);
            self.draw_launch_buttons(ui, ctx);
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            self.draw_project_picker(ui);
            ui.add_space(8.0);
            self.draw_status(ui);
        });
    }
}

impl LauncherApp {
    // -----------------------------------------------------------------------
    // Drawing
    // -----------------------------------------------------------------------

    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("⚒  NovaForge Workspace")
                    .size(26.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new("The development platform for Nova-Forge")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(160, 160, 180)),
            );
            if let Some(ref p) = self.project {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("Project: {}", p.project_name))
                        .size(12.0)
                        .color(egui::Color32::from_rgb(130, 200, 130)),
                );
            }
        });
    }

    fn draw_launch_buttons(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let btn_size = egui::vec2(160.0, 56.0);

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                // Centre the row manually.
                let total = btn_size.x * 3.0 + ui.spacing().item_spacing.x * 2.0;
                let offset = (ui.available_width() - total) / 2.0;
                ui.add_space(offset.max(0.0));

                if ui
                    .add_sized(btn_size, egui::Button::new("▶  Play"))
                    .on_hover_text("Launch the Nova-Forge game client")
                    .clicked()
                {
                    self.launch_play();
                }

                if ui
                    .add_sized(btn_size, egui::Button::new("🌐  Host LAN"))
                    .on_hover_text("Start a LAN server and join it")
                    .clicked()
                {
                    self.launch_server();
                }

                if ui
                    .add_sized(btn_size, egui::Button::new("🔧  Open Workspace"))
                    .on_hover_text("Open the full NovaForge editor suite")
                    .clicked()
                {
                    self.open_editors(ctx);
                }
            });
        });
    }

    fn draw_project_picker(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Project file").strong());
        ui.horizontal(|ui| {
            let input = ui.add(
                egui::TextEdit::singleline(&mut self.project_path_input)
                    .hint_text(format!("Path to folder or {MANIFEST_FILE}"))
                    .desired_width(f32::INFINITY),
            );
            if input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.load_project();
            }
            if ui.button("Open").clicked() {
                self.load_project();
            }
        });

        if !self.recent_projects.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Recent projects").size(11.0));
            let mut open_path: Option<PathBuf> = None;
            for path in &self.recent_projects {
                if ui
                    .small_button(path.display().to_string())
                    .clicked()
                {
                    open_path = Some(path.clone());
                }
            }
            if let Some(path) = open_path {
                self.project_path_input = path.display().to_string();
                self.load_project();
            }
        }
    }

    fn draw_status(&self, ui: &mut egui::Ui) {
        if !self.status.is_empty() {
            ui.separator();
            ui.label(
                egui::RichText::new(&self.status)
                    .size(11.0)
                    .color(egui::Color32::from_rgb(180, 180, 200)),
            );
        }
    }

    // -----------------------------------------------------------------------
    // Actions
    // -----------------------------------------------------------------------

    fn launch_play(&mut self) {
        match &self.project {
            Some(project) => {
                let bin = project.nova_forge_binary();
                match std::process::Command::new(&bin).spawn() {
                    Ok(_) => self.status = format!("Launched: {}", bin.display()),
                    Err(e) => self.status = format!("Error launching game: {e}"),
                }
            }
            None => {
                self.status =
                    "No project loaded. Open a novaforge.workspace.toml first.".to_string();
            }
        }
    }

    fn launch_server(&mut self) {
        match &self.project {
            Some(project) => {
                let script = project.build_script();
                match std::process::Command::new(&script).arg("server").spawn() {
                    Ok(_) => self.status = "Server started.".to_string(),
                    Err(e) => self.status = format!("Error launching server: {e}"),
                }
            }
            None => {
                self.status =
                    "No project loaded. Open a novaforge.workspace.toml first.".to_string();
            }
        }
    }

    fn open_editors(&mut self, _ctx: &egui::Context) {
        // Resolve sibling `novaforge-editors` binary next to this executable.
        let editors_bin = std::env::current_exe().ok().and_then(|exe| {
            exe.parent().map(|dir| {
                #[cfg(target_os = "windows")]
                let name = "novaforge-editors.exe";
                #[cfg(not(target_os = "windows"))]
                let name = "novaforge-editors";
                dir.join(name)
            })
        });

        match editors_bin {
            Some(bin) if bin.exists() => match std::process::Command::new(&bin).spawn() {
                Ok(_) => self.status = "Editor suite launched.".to_string(),
                Err(e) => self.status = format!("Error: {e}"),
            },
            _ => {
                self.status =
                    "novaforge-editors not found. Run `cargo run -p novaforge-editors` directly."
                        .to_string();
            }
        }
    }

    fn load_project(&mut self) {
        use std::path::Path;
        let path = Path::new(self.project_path_input.trim());
        if path.as_os_str().is_empty() {
            self.status = "Please enter a path.".to_string();
            return;
        }
        match WorkspaceManifest::load(path) {
            Ok(manifest) => {
                self.status = format!("Opened: {}", manifest.project_name);
                // Track in recent list.
                let canonical = path
                    .canonicalize()
                    .unwrap_or_else(|_| path.to_path_buf());
                self.recent_projects.retain(|p| p != &canonical);
                self.recent_projects.insert(0, canonical);
                self.recent_projects.truncate(5);
                self.project = Some(manifest);
            }
            Err(e) => {
                self.status = format!("Failed to load project: {e}");
            }
        }
    }
}
