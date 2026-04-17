//! Shared UI primitives and the [`EditorPanel`] trait for NovaForge Workspace.

/// Context passed to every panel on each frame.
///
/// The master editor builds this from the loaded [`WorkspaceManifest`] and
/// hands it to each panel's [`EditorPanel::ui`] call.
#[derive(Default)]
pub struct PanelContext {
    /// Human-readable project name, or `None` when no project is open.
    pub project_name: Option<String>,
    /// Absolute path to the Nova-Forge repository root, or `None`.
    pub nova_forge_path: Option<std::path::PathBuf>,
    /// Absolute path to the asset root directory, or `None`.
    pub asset_root: Option<std::path::PathBuf>,
}

/// Every editor panel implements this trait.
///
/// Panels are library crates; the master `novaforge-editors` binary hosts them
/// all inside an [`egui_dock`](https://docs.rs/egui_dock) layout.  Each panel
/// can also run standalone by enabling its `standalone` Cargo feature.
pub trait EditorPanel {
    /// Short human-readable name shown in the docked tab header.
    fn title(&self) -> &str;

    /// Draw the panel contents into `ui`.
    ///
    /// Called once per frame while the panel's tab is visible.
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext);

    /// Optional per-frame update hook called even when the panel is hidden.
    ///
    /// Use this for background polling (e.g. reading build log channel).
    fn background_update(&mut self) {}
}

/// A simple coloured separator with a label — shared across panels.
pub fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.add_space(4.0);
    ui.separator();
    ui.label(egui::RichText::new(label).strong().size(12.0));
    ui.add_space(2.0);
}

/// Placeholder viewport / canvas widget drawn as a dark rounded rectangle.
///
/// Returns the [`egui::Response`] for the allocated area.
pub fn placeholder_viewport(ui: &mut egui::Ui, label: &str, icon: &str) -> egui::Response {
    let available = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(25, 25, 30));
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 70)),
            egui::StrokeKind::Middle,
        );
        painter.text(
            rect.center() - egui::vec2(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(32.0),
            egui::Color32::from_rgb(80, 80, 100),
        );
        painter.text(
            rect.center() + egui::vec2(0.0, 26.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(13.0),
            egui::Color32::from_rgb(90, 90, 110),
        );
    }

    response
}
