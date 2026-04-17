//! Standalone launcher for the Build Tool panel.
//! Compile with: `cargo run -p editor-build --features standalone`

use editor_build::BuildToolPanel;
use eframe::egui;
use novaforge_ui::{EditorPanel, PanelContext};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Build Tool — NovaForge Workspace",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(StandaloneApp::new(BuildToolPanel::default())))),
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
