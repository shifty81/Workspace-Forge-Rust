//! NovaForge Workspace — master editor application.
//!
//! Opens a dockable multi-panel editor suite.  All ten tool panels are hosted
//! in a single window using [`egui_dock`].  Every panel can be detached and
//! re-docked interactively.
//!
//! Run with: `cargo run -p novaforge-editors`

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use novaforge_ai::{StubAI, WorkspaceAI};
use novaforge_build::BuildCommand;
use novaforge_project::{AssetKind, WorkspaceManifest, MANIFEST_FILE};

// Editor panel imports
use editor_animation::AnimationEditor;
use editor_asset::AssetEditor;
use editor_build::BuildToolPanel;
use editor_data::DataEditor;
use editor_gamefile::GameFileEditor;
use editor_material::MaterialEditor;
use editor_scene::SceneEditor;
use editor_ui::UiEditorPanel;
use editor_vlogic::VLogicEditor;

use novaforge_ui::{EditorPanel, PanelContext};
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "NovaForge Workspace — Editor Suite",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1600.0, 960.0])
                .with_title("NovaForge Workspace — Editor Suite"),
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        },
        Box::new(|cc| {
            // Apply dark visuals immediately so the very first frame is dark.
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            // Initialise the wgpu 3-D viewport pipeline if the wgpu backend is
            // available (it always is with eframe's default renderer).
            if let Some(render_state) = cc.wgpu_render_state.as_ref() {
                editor_viewport::init_viewport_pipeline(render_state);
            }
            Ok(Box::new(EditorApp::new()))
        }),
    )
}

// ---------------------------------------------------------------------------
// Tab identifier
// ---------------------------------------------------------------------------

/// Identifies each dockable panel.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Tab {
    WorkspaceBrowser,
    Scene,
    Asset,
    Material,
    VLogic,
    Ui,
    Animation,
    Data,
    Build,
    AiTool,
    GameFile,
}

impl Tab {
    fn label(&self) -> &str {
        match self {
            Tab::WorkspaceBrowser => "📁 Workspace",
            Tab::Scene => "🌐 Scene",
            Tab::Asset => "🖼 Assets",
            Tab::Material => "🎨 Material",
            Tab::VLogic => "🔗 V-Logic",
            Tab::Ui => "📐 UI",
            Tab::Animation => "🎬 Animation",
            Tab::Data => "📋 Data",
            Tab::Build => "🔨 Build",
            Tab::AiTool => "🤖 AI Tool",
            Tab::GameFile => "📝 File Editor",
        }
    }
}

// ---------------------------------------------------------------------------
// All panels, owned together
// ---------------------------------------------------------------------------

struct Panels {
    workspace_browser: WorkspaceBrowser,
    scene: SceneEditor,
    asset: AssetEditor,
    material: MaterialEditor,
    vlogic: VLogicEditor,
    ui_editor: UiEditorPanel,
    animation: AnimationEditor,
    data: DataEditor,
    build: BuildToolPanel,
    ai_tool: AiToolPanel,
    game_file: GameFileEditor,
}

impl Panels {
    fn new() -> Self {
        Self {
            workspace_browser: WorkspaceBrowser::default(),
            scene: SceneEditor::new(),
            asset: AssetEditor::new(),
            material: MaterialEditor::default(),
            vlogic: VLogicEditor::default(),
            ui_editor: UiEditorPanel::default(),
            animation: AnimationEditor::default(),
            data: DataEditor::default(),
            build: BuildToolPanel::default(),
            ai_tool: AiToolPanel::new(),
            game_file: GameFileEditor::default(),
        }
    }

    fn background_update_all(&mut self) {
        self.animation.background_update();
        self.build.background_update();
    }
}

// ---------------------------------------------------------------------------
// Tab viewer (delegates to the correct panel)
// ---------------------------------------------------------------------------

struct EditorTabViewer<'a> {
    panels: &'a mut Panels,
    ctx: &'a PanelContext,
}

impl TabViewer for EditorTabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.label().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        let ctx = self.ctx;
        let p = &mut self.panels;
        match tab {
            Tab::WorkspaceBrowser => p.workspace_browser.ui(ui, ctx),
            Tab::Scene => p.scene.ui(ui, ctx),
            Tab::Asset => p.asset.ui(ui, ctx),
            Tab::Material => p.material.ui(ui, ctx),
            Tab::VLogic => p.vlogic.ui(ui, ctx),
            Tab::Ui => p.ui_editor.ui(ui, ctx),
            Tab::Animation => p.animation.ui(ui, ctx),
            Tab::Data => p.data.ui(ui, ctx),
            Tab::Build => p.build.ui(ui, ctx),
            Tab::AiTool => p.ai_tool.ui(ui, ctx),
            Tab::GameFile => p.game_file.ui(ui, ctx),
        }
    }

    fn closeable(&mut self, _tab: &mut Tab) -> bool {
        // Panels can be closed (toggled back from the View menu).
        true
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/// Editor colour theme.  Defaults to [`Theme::Dark`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    fn visuals(self) -> egui::Visuals {
        match self {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => egui::Visuals::light(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Theme::Dark => "🌙 Dark",
            Theme::Light => "☀ Light",
        }
    }
}

// ---------------------------------------------------------------------------
// Master editor app
// ---------------------------------------------------------------------------

struct EditorApp {
    dock_state: DockState<Tab>,
    panels: Panels,
    project: Option<WorkspaceManifest>,
    project_path_input: String,
    panel_ctx: PanelContext,
    status: String,
    theme: Theme,
    /// Paths of the last 5 successfully opened projects (most-recent first).
    recent_projects: Vec<String>,
}

impl EditorApp {
    fn new() -> Self {
        // Build the initial docking layout:
        //   left sidebar: Workspace Browser
        //   centre (main):  Scene | Asset | Material | V-Logic | UI | Animation | Data
        //   bottom strip:   Build  |  AI Tool
        let mut dock_state = DockState::new(vec![Tab::Scene]);

        let [centre, _left] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.18,
            vec![Tab::WorkspaceBrowser],
        );

        let [_centre, _bottom] =
            dock_state
                .main_surface_mut()
                .split_below(centre, 0.72, vec![Tab::Build, Tab::AiTool]);

        // Add the remaining panels as tabs in the centre area.
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(Tab::Asset);
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(Tab::Material);
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(Tab::VLogic);
        dock_state.main_surface_mut().push_to_focused_leaf(Tab::Ui);
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(Tab::Animation);
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(Tab::Data);
        dock_state
            .main_surface_mut()
            .push_to_focused_leaf(Tab::GameFile);

        Self {
            dock_state,
            panels: Panels::new(),
            project: None,
            project_path_input: String::new(),
            panel_ctx: PanelContext::default(),
            status: "No project loaded.".to_string(),
            theme: Theme::Dark,
            recent_projects: Self::load_recent(),
        }
    }

    fn all_tabs() -> &'static [Tab] {
        &[
            Tab::WorkspaceBrowser,
            Tab::Scene,
            Tab::Asset,
            Tab::Material,
            Tab::VLogic,
            Tab::Ui,
            Tab::Animation,
            Tab::Data,
            Tab::Build,
            Tab::AiTool,
            Tab::GameFile,
        ]
    }

    // -----------------------------------------------------------------------
    // Persistent recent projects
    // -----------------------------------------------------------------------

    /// Platform-specific path to the recent-projects config file:
    /// - Linux/macOS: `$HOME/.config/novaforge-workspace/recent.toml`
    /// - Windows:     `%APPDATA%\novaforge-workspace\recent.toml`
    fn config_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        let base = std::env::var("APPDATA").ok().map(PathBuf::from);
        #[cfg(not(target_os = "windows"))]
        let base = std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config"));
        base.map(|b| b.join("novaforge-workspace").join("recent.toml"))
    }

    /// Load the recent-projects list from disk.  Returns an empty list on any
    /// error (missing file, parse failure, …) so startup is never blocked.
    fn load_recent() -> Vec<String> {
        let Some(path) = Self::config_path() else {
            return Vec::new();
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        #[derive(serde::Deserialize)]
        struct RecentFile {
            recent: Vec<String>,
        }
        toml::from_str::<RecentFile>(&content)
            .map(|f| f.recent)
            .unwrap_or_default()
    }

    /// Persist the current recent-projects list to disk.  Silently ignores errors.
    fn save_recent(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        #[derive(serde::Serialize)]
        struct RecentFile<'a> {
            recent: &'a [String],
        }
        if let Ok(content) = toml::to_string_pretty(&RecentFile {
            recent: &self.recent_projects,
        }) {
            let _ = std::fs::write(&path, content);
        }
    }

    fn toggle_tab(&mut self, tab: Tab) {
        if let Some(index) = self.dock_state.find_tab(&tab) {
            self.dock_state.remove_tab(index);
        } else {
            self.dock_state.push_to_focused_leaf(tab);
        }
    }

    fn ensure_tab_open(&mut self, tab: Tab) {
        if self.dock_state.find_tab(&tab).is_none() {
            self.dock_state.push_to_focused_leaf(tab);
        }
    }

    fn load_project(&mut self) {
        use std::path::Path;
        let path_str = self.project_path_input.trim().to_string();
        if path_str.is_empty() {
            self.status = "Please enter a project path.".to_string();
            return;
        }
        match WorkspaceManifest::load(Path::new(&path_str)) {
            Ok(manifest) => {
                self.panel_ctx = PanelContext {
                    project_name: Some(manifest.project_name.clone()),
                    nova_forge_path: Some(manifest.nova_forge_path.clone()),
                    asset_root: Some(manifest.asset_root.clone()),
                    selected_file: None,
                };
                self.status = format!("Project: {}", manifest.project_name);
                self.panels
                    .workspace_browser
                    .set_root(manifest.asset_root.clone());
                self.project = Some(manifest);
                // Record in recent projects (most-recent first, no duplicates).
                self.recent_projects.retain(|p| p != &path_str);
                self.recent_projects.insert(0, path_str);
                self.recent_projects.truncate(5);
                // Persist to disk so the list survives restarts.
                self.save_recent();
            }
            Err(e) => {
                self.status = format!("Error: {e}");
            }
        }
    }

    // -----------------------------------------------------------------------
    // Menu bar
    // -----------------------------------------------------------------------

    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Project:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.project_path_input)
                                .hint_text(format!("Path to {MANIFEST_FILE}"))
                                .desired_width(260.0),
                        );
                        if ui
                            .button("📂 Browse…")
                            .on_hover_text("Open a file browser to locate the workspace manifest")
                            .clicked()
                        {
                            // Build a suggested starting directory from whatever the
                            // user has already typed so the dialog opens nearby.
                            let start_dir = {
                                let p = std::path::Path::new(self.project_path_input.trim());
                                if p.is_dir() {
                                    Some(p.to_path_buf())
                                } else {
                                    p.parent().map(|d| d.to_path_buf())
                                }
                            };

                            let mut dialog = rfd::FileDialog::new()
                                .add_filter("NovaForge workspace manifest", &["toml"])
                                .add_filter("All files", &["*"])
                                .set_title("Open NovaForge Workspace");

                            if let Some(dir) = start_dir {
                                dialog = dialog.set_directory(dir);
                            }

                            if let Some(path) = dialog.pick_file() {
                                self.project_path_input = path.to_string_lossy().to_string();
                                self.load_project();
                                ui.close_menu();
                            }
                        }
                        if ui.button("Open").clicked() {
                            self.load_project();
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if ui.button("Save Project").clicked() {
                        if let Some(ref m) = self.project {
                            let path = PathBuf::from(&self.project_path_input);
                            match m.save(&path) {
                                Ok(()) => self.status = "Project saved.".to_string(),
                                Err(e) => self.status = format!("Save error: {e}"),
                            }
                        }
                        ui.close_menu();
                    }
                    // Recent Projects submenu (only shown when there is history).
                    if !self.recent_projects.is_empty() {
                        ui.separator();
                        ui.menu_button("Recent Projects", |ui| {
                            // Collect the list to avoid a mutable + immutable borrow conflict.
                            let recents = self.recent_projects.clone();
                            let mut open_path: Option<String> = None;
                            for path in &recents {
                                // Shorten long paths for display using char-aware iteration.
                                let chars: Vec<char> = path.chars().collect();
                                let display = if chars.len() > 60 {
                                    let tail: String = chars[chars.len() - 57..].iter().collect();
                                    format!("…{tail}")
                                } else {
                                    path.clone()
                                };
                                if ui.button(display).on_hover_text(path).clicked() {
                                    open_path = Some(path.clone());
                                    ui.close_menu();
                                }
                            }
                            if let Some(path) = open_path {
                                self.project_path_input = path;
                                self.load_project();
                            }
                        });
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.label(egui::RichText::new("Toggle panels").weak());
                    ui.separator();
                    for tab in Self::all_tabs() {
                        let visible = self.dock_state.find_tab(tab).is_some();
                        let mut v = visible;
                        if ui.checkbox(&mut v, tab.label()).changed() {
                            self.toggle_tab(tab.clone());
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Theme").weak());
                    for t in [Theme::Dark, Theme::Light] {
                        if ui.radio(self.theme == t, t.label()).clicked() {
                            self.theme = t;
                        }
                    }
                    ui.separator();
                    if ui.button("Reset Layout").clicked() {
                        let theme = self.theme;
                        let recents = self.recent_projects.clone();
                        *self = EditorApp::new();
                        self.theme = theme;
                        self.recent_projects = recents;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Build", |ui| {
                    let nova_path = self.panel_ctx.nova_forge_path.clone();
                    if ui
                        .button("🔨 Build (debug)")
                        .on_hover_text("cargo build  [Ctrl+B]")
                        .clicked()
                    {
                        self.ensure_tab_open(Tab::Build);
                        self.panels
                            .build
                            .trigger(BuildCommand::Build, nova_path.as_ref());
                        ui.close_menu();
                    }
                    if ui
                        .button("🚀 Build (release)")
                        .on_hover_text("cargo build --release")
                        .clicked()
                    {
                        self.ensure_tab_open(Tab::Build);
                        self.panels
                            .build
                            .trigger(BuildCommand::Release, nova_path.as_ref());
                        ui.close_menu();
                    }
                    if ui
                        .button("▶ Run")
                        .on_hover_text("Build and run the client  [Ctrl+R]")
                        .clicked()
                    {
                        self.ensure_tab_open(Tab::Build);
                        self.panels
                            .build
                            .trigger(BuildCommand::Run, nova_path.as_ref());
                        ui.close_menu();
                    }
                    if ui.button("🧪 Test").on_hover_text("cargo test").clicked() {
                        self.ensure_tab_open(Tab::Build);
                        self.panels
                            .build
                            .trigger(BuildCommand::Test, nova_path.as_ref());
                        ui.close_menu();
                    }
                });

                ui.menu_button("AI", |ui| {
                    ui.label("AI provider: Offline (stub)");
                    ui.separator();
                    if ui.button("Open AI Tool").clicked() {
                        self.ensure_tab_open(Tab::AiTool);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    ui.hyperlink_to(
                        "NovaForge Workspace on GitHub",
                        "https://github.com/shifty81/Workspace-Forge-Rust",
                    );
                    ui.hyperlink_to(
                        "Nova-Forge on GitHub",
                        "https://github.com/shifty81/Nova-Forge",
                    );
                });
            });
        });
    }

    // -----------------------------------------------------------------------
    // Status bar
    // -----------------------------------------------------------------------

    fn show_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Project: {}",
                    self.panel_ctx.project_name.as_deref().unwrap_or("None")
                ));
                ui.separator();
                ui.label(&self.status);
                ui.separator();
                ui.label("AI: Offline (stub)");
            });
        });
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keep visuals in sync with the chosen theme (cheap — egui deduplicates).
        ctx.set_visuals(self.theme.visuals());

        // Background updates (build log polling, animation playhead)
        self.panels.background_update_all();

        // Propagate workspace browser file selection to the panel context so
        // the Game File Editor can auto-open the chosen file.
        self.panel_ctx.selected_file = self.panels.workspace_browser.selected_absolute_path();

        // If the browser context menu requested "Open in File Editor", ensure
        // that tab is visible so the user sees the file immediately.
        if self
            .panels
            .workspace_browser
            .open_file_request
            .take()
            .is_some()
        {
            self.ensure_tab_open(Tab::GameFile);
        }

        // If the Asset Editor requested to open a file (double-click / context
        // menu / "Open in File Editor" button), route it to the Game File Editor.
        if let Some(path) = self.panels.asset.open_file_request.take() {
            self.panel_ctx.selected_file = Some(path);
            self.ensure_tab_open(Tab::GameFile);
        }

        // ── Keyboard shortcuts ────────────────────────────────────────────────
        // Ctrl+S — save the currently open file in the Game File Editor.
        // Ctrl+B — trigger a debug build.
        // Ctrl+R — run the game client.
        let (ctrl_s, ctrl_b, ctrl_r) = ctx.input(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
            (
                ctrl && i.key_pressed(egui::Key::S),
                ctrl && i.key_pressed(egui::Key::B),
                ctrl && i.key_pressed(egui::Key::R),
            )
        });
        if ctrl_s {
            self.panels.game_file.save_if_dirty();
        }
        if ctrl_b {
            let nova_path = self.panel_ctx.nova_forge_path.clone();
            self.ensure_tab_open(Tab::Build);
            self.panels
                .build
                .trigger(BuildCommand::Build, nova_path.as_ref());
        }
        if ctrl_r {
            let nova_path = self.panel_ctx.nova_forge_path.clone();
            self.ensure_tab_open(Tab::Build);
            self.panels
                .build
                .trigger(BuildCommand::Run, nova_path.as_ref());
        }

        self.show_menu_bar(ctx);
        self.show_status_bar(ctx);

        // Dock area fills the remaining space
        DockArea::new(&mut self.dock_state).show(
            ctx,
            &mut EditorTabViewer {
                panels: &mut self.panels,
                ctx: &self.panel_ctx,
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Workspace Browser (defined here, not a separate crate)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct WorkspaceBrowser {
    root: Option<PathBuf>,
    entries: Vec<BrowserEntry>,
    filter: String,
    selected: Option<usize>,
    /// Relative paths of directories that are currently collapsed.
    collapsed: std::collections::HashSet<String>,
    /// Set by the context menu when the user chooses "Open in File Editor".
    /// The main app reads and clears this every frame.
    open_file_request: Option<PathBuf>,
}

#[derive(Clone)]
struct BrowserEntry {
    display: String,
    /// Relative path from the asset root (forward-slash separated).
    path: String,
    is_dir: bool,
    depth: usize,
    /// Icon string inferred from the file extension for non-directory entries.
    icon: &'static str,
}

impl WorkspaceBrowser {
    fn set_root(&mut self, root: PathBuf) {
        self.entries = scan_dir(&root, &root, 0);
        self.root = Some(root);
        self.selected = None;
        // Reset collapse state — paths are relative to the root, so stale
        // collapsed entries from a previous project would be meaningless.
        self.collapsed.clear();
    }

    /// Returns the absolute path of the currently selected **file** entry,
    /// or `None` when nothing is selected or a directory is selected.
    fn selected_absolute_path(&self) -> Option<PathBuf> {
        let root = self.root.as_ref()?;
        let idx = self.selected?;
        let entry = self.entries.get(idx)?;
        if entry.is_dir {
            return None;
        }
        // entry.path uses forward slashes; join handles cross-platform.
        Some(root.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR)))
    }

    /// Returns `true` if any ancestor directory of `path` is in the collapsed
    /// set.  `path` is a relative, forward-slash separated string.
    fn is_ancestor_collapsed(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('/').collect();
        for i in 0..parts.len().saturating_sub(1) {
            let ancestor = parts[..=i].join("/");
            if self.collapsed.contains(&ancestor) {
                return true;
            }
        }
        false
    }
}

fn scan_dir(path: &PathBuf, root: &PathBuf, depth: usize) -> Vec<BrowserEntry> {
    let mut entries = Vec::new();
    if depth > 4 {
        return entries;
    }
    let Ok(read) = std::fs::read_dir(path) else {
        return entries;
    };
    let mut items: Vec<std::fs::DirEntry> = read.flatten().collect();
    items.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));
    for item in items {
        let p = item.path();
        let name = item.file_name().to_string_lossy().to_string();
        let is_dir = p.is_dir();
        let rel_path = p
            .strip_prefix(root)
            .map(|r| r.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| name.clone());
        let icon = if is_dir {
            "📁"
        } else {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            AssetKind::from_extension(ext).icon()
        };
        entries.push(BrowserEntry {
            display: name,
            path: rel_path,
            is_dir,
            depth,
            icon,
        });
        if is_dir {
            entries.extend(scan_dir(&p, root, depth + 1));
        }
    }
    entries
}

impl EditorPanel for WorkspaceBrowser {
    fn title(&self) -> &str {
        "Workspace"
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("Filter…")
                    .desired_width(f32::INFINITY),
            );
            if ui
                .small_button("⟳")
                .on_hover_text("Refresh file tree")
                .clicked()
            {
                if let Some(root) = self.root.clone().or_else(|| ctx.asset_root.clone()) {
                    self.set_root(root);
                }
            }
        });

        if let Some(ref root) = self.root {
            ui.label(
                egui::RichText::new(root.display().to_string())
                    .size(10.0)
                    .color(egui::Color32::from_rgb(120, 120, 140)),
            );
        } else if let Some(ref root) = ctx.asset_root {
            // Auto-populate from project context if not yet set.
            let root = root.clone();
            self.set_root(root);
        }

        ui.separator();

        let filter_lower = self.filter.to_lowercase();

        // Collect toggle actions outside the borrow on self.entries to avoid
        // a simultaneous mutable + immutable borrow conflict.
        let mut toggle_path: Option<String> = None;
        let mut new_selected = self.selected;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, entry) in self.entries.iter().enumerate() {
                // Hide entries whose parent directory is collapsed.
                if self.is_ancestor_collapsed(&entry.path) {
                    continue;
                }

                // When filtering, show only entries whose name matches (and
                // always show directories so the tree structure is preserved).
                if !filter_lower.is_empty()
                    && !entry.is_dir
                    && !entry.display.to_lowercase().contains(&filter_lower)
                {
                    continue;
                }

                let indent = entry.depth as f32 * 14.0;

                if entry.is_dir {
                    let is_collapsed = self.collapsed.contains(&entry.path);
                    let arrow = if is_collapsed { "▸" } else { "▾" };
                    ui.horizontal(|ui| {
                        ui.add_space(indent);
                        if ui
                            .small_button(format!("{arrow} 📁 {}", entry.display))
                            .clicked()
                        {
                            toggle_path = Some(entry.path.clone());
                        }
                    });
                } else {
                    let label = format!("{} {}", entry.icon, entry.display);
                    let selected = self.selected == Some(i);

                    // Build the absolute path once for the context menu.
                    let abs_path = self
                        .root
                        .as_ref()
                        .map(|r| r.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR)));

                    ui.horizontal(|ui| {
                        ui.add_space(indent);
                        let response = ui.selectable_label(selected, label);
                        if response.clicked() {
                            new_selected = Some(i);
                        }
                        // Right-click context menu for file entries.
                        response.context_menu(|ui| {
                            if ui
                                .button("📝 Open in File Editor")
                                .on_hover_text("Open this file in the Game File Editor panel")
                                .clicked()
                            {
                                new_selected = Some(i);
                                if let Some(ref p) = abs_path {
                                    self.open_file_request = Some(p.clone());
                                }
                                ui.close_menu();
                            }
                            if ui
                                .button("📋 Copy Path")
                                .on_hover_text("Copy the absolute file path to the clipboard")
                                .clicked()
                            {
                                if let Some(ref p) = abs_path {
                                    ui.ctx().copy_text(p.display().to_string());
                                }
                                ui.close_menu();
                            }
                        });
                    });
                }
            }

            if self.entries.is_empty() && self.root.is_some() {
                ui.label(
                    egui::RichText::new("Directory is empty or inaccessible.")
                        .italics()
                        .color(egui::Color32::from_rgb(120, 120, 140)),
                );
            } else if self.root.is_none() {
                ui.label(
                    egui::RichText::new("Open a project to browse its files.")
                        .italics()
                        .color(egui::Color32::from_rgb(120, 120, 140)),
                );
            }
        });

        // Apply the collapse / expand toggle after the borrow on entries ends.
        if let Some(path) = toggle_path {
            if self.collapsed.contains(&path) {
                self.collapsed.remove(&path);
            } else {
                self.collapsed.insert(path);
            }
        }

        self.selected = new_selected;
    }
}

// ---------------------------------------------------------------------------
// AI Tool panel (defined here, not a separate crate)
// ---------------------------------------------------------------------------

struct AiToolPanel {
    ai: Box<dyn WorkspaceAI>,
    input: String,
    history: Vec<(Role, String)>,
}

#[derive(Clone, Copy)]
enum Role {
    User,
    Assistant,
}

impl AiToolPanel {
    fn new() -> Self {
        Self {
            ai: Box::new(StubAI),
            input: String::new(),
            history: vec![(
                Role::Assistant,
                "NovaForge AI is offline (stub). Configure a provider to enable AI features."
                    .to_string(),
            )],
        }
    }

    fn submit(&mut self) {
        let prompt = self.input.trim().to_string();
        if prompt.is_empty() {
            return;
        }
        self.history.push((Role::User, prompt.clone()));
        self.input.clear();

        // The stub returns immediately, so we can block.
        let response = futures::executor::block_on(self.ai.query(&prompt));
        self.history.push((Role::Assistant, response));
    }
}

impl EditorPanel for AiToolPanel {
    fn title(&self) -> &str {
        "AI Tool"
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
        // Status badge
        ui.horizontal(|ui| {
            let (colour, label) = if self.ai.is_available() {
                (egui::Color32::from_rgb(80, 200, 80), "● Online")
            } else {
                (egui::Color32::from_rgb(200, 80, 80), "● Offline")
            };
            ui.label(egui::RichText::new(label).color(colour));
            ui.label(format!("Provider: {}", self.ai.provider_name()));
        });

        ui.separator();

        // Chat history
        let history_height = ui.available_height() - 48.0;
        egui::ScrollArea::vertical()
            .max_height(history_height.max(40.0))
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (role, msg) in &self.history {
                    match role {
                        Role::User => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("You: ")
                                        .strong()
                                        .color(egui::Color32::from_rgb(140, 180, 240)),
                                );
                                ui.label(msg);
                            });
                        }
                        Role::Assistant => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("AI: ")
                                        .strong()
                                        .color(egui::Color32::from_rgb(180, 220, 160)),
                                );
                                ui.label(msg);
                            });
                        }
                    }
                    ui.add_space(2.0);
                }
            });

        ui.separator();

        // Input row
        ui.horizontal(|ui| {
            let input_widget = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("Ask the AI…")
                    .desired_width(f32::INFINITY),
            );
            let send = ui.button("Send");
            if send.clicked()
                || (input_widget.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                self.submit();
            }
        });
    }
}
