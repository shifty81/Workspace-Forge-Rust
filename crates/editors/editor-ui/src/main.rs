//! Standalone launcher for the UI Layout Editor.
//! Compile with: `cargo run -p editor-ui --features standalone`

use editor_ui::UiEditorPanel;
use eframe::egui;
use novaforge_ui::{EditorPanel, PanelContext};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "UI Editor — NovaForge Workspace",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(StandaloneApp::new(UiEditorPanel::default())))),
    )
}

struct StandaloneApp<P: EditorPanel> {
    panel: P,
    ctx: PanelContext,
}

impl<P: EditorPanel> StandaloneApp<P> {
    fn new(panel: P) -> Self {
        Self {
            panel,
            ctx: PanelContext::default(),
        }
    }
}

impl<P: EditorPanel> eframe::App for StandaloneApp<P> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.panel.background_update();
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.label(egui::RichText::new(self.panel.title()).strong().size(14.0));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.panel.ui(ui, &self.ctx);
        });
    }
}
