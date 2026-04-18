//! Standalone launcher for the Game File Editor.
//! Compile with: `cargo run -p editor-gamefile --features standalone`

use editor_gamefile::GameFileEditor;
use eframe::egui;
use novaforge_ui::{EditorPanel, PanelContext};

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Game File Editor — NovaForge Workspace",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 700.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(StandaloneApp {
                panel: GameFileEditor::default(),
                ctx: PanelContext::default(),
            }))
        }),
    )
}

struct StandaloneApp {
    panel: GameFileEditor,
    ctx: PanelContext,
}

impl eframe::App for StandaloneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.label(egui::RichText::new(self.panel.title()).strong().size(14.0));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.panel.ui(ui, &self.ctx);
        });
    }
}
