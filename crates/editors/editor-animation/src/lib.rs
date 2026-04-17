//! Animation Timeline & State Machine Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};

/// A single keyframe on the timeline.
#[derive(Clone)]
#[allow(dead_code)]
struct Keyframe {
    time: f32,
    label: String,
}

/// An animation track.
#[derive(Clone)]
struct Track {
    name: String,
    keyframes: Vec<Keyframe>,
}

/// Animation Editor panel.
pub struct AnimationEditor {
    tracks: Vec<Track>,
    playhead: f32,
    duration: f32,
    playing: bool,
    zoom: f32,
}

impl Default for AnimationEditor {
    fn default() -> Self {
        Self {
            tracks: vec![
                Track {
                    name: "Root Bone".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.0,
                            label: "A".to_string(),
                        },
                        Keyframe {
                            time: 0.5,
                            label: "B".to_string(),
                        },
                        Keyframe {
                            time: 1.0,
                            label: "C".to_string(),
                        },
                    ],
                },
                Track {
                    name: "Weapon".to_string(),
                    keyframes: vec![
                        Keyframe {
                            time: 0.2,
                            label: "A".to_string(),
                        },
                        Keyframe {
                            time: 0.8,
                            label: "B".to_string(),
                        },
                    ],
                },
                Track {
                    name: "FX".to_string(),
                    keyframes: vec![Keyframe {
                        time: 0.6,
                        label: "A".to_string(),
                    }],
                },
            ],
            playhead: 0.0,
            duration: 2.0,
            playing: false,
            zoom: 1.0,
        }
    }
}

impl EditorPanel for AnimationEditor {
    fn title(&self) -> &str {
        "Animation Editor"
    }

    fn background_update(&mut self) {
        if self.playing {
            // Advance playhead at ~60 fps equivalent (1/60 s per frame).
            self.playhead += 1.0 / 60.0;
            if self.playhead >= self.duration {
                self.playhead = 0.0;
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &PanelContext) {
        // Transport bar
        ui.horizontal(|ui| {
            if ui.button("⏮ Start").clicked() {
                self.playhead = 0.0;
            }
            let play_label = if self.playing {
                "⏸ Pause"
            } else {
                "▶ Play"
            };
            if ui.button(play_label).clicked() {
                self.playing = !self.playing;
            }
            if ui.button("⏹ Stop").clicked() {
                self.playing = false;
                self.playhead = 0.0;
            }
            ui.separator();
            ui.label(format!(
                "Time: {:.2} / {:.2} s",
                self.playhead, self.duration
            ));
            ui.separator();
            ui.label("Zoom:");
            if ui.button("＋").clicked() {
                self.zoom = (self.zoom * 1.2).min(8.0);
            }
            if ui.button("−").clicked() {
                self.zoom = (self.zoom / 1.2).max(0.25);
            }
        });

        ui.separator();

        let label_w = 100.0_f32;
        let row_h = 24.0_f32;
        let timeline_w = ui.available_width() - label_w;
        let total_h = (self.tracks.len() as f32 + 1.0) * row_h;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(label_w + timeline_w, total_h),
                egui::Sense::click(),
            );

            if !ui.is_rect_visible(rect) {
                return;
            }

            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 22, 28));

            // Time ruler
            let ruler_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(label_w, 0.0),
                egui::vec2(timeline_w, row_h),
            );
            painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(35, 35, 44));

            let px_per_sec = timeline_w / self.duration * self.zoom;
            let tick_step = if self.zoom > 2.0 {
                0.1
            } else if self.zoom > 0.5 {
                0.5
            } else {
                1.0
            };
            let mut t = 0.0_f32;
            while t <= self.duration {
                let x = ruler_rect.left() + t * px_per_sec;
                if x > ruler_rect.right() {
                    break;
                }
                painter.line_segment(
                    [
                        egui::pos2(x, ruler_rect.top()),
                        egui::pos2(x, ruler_rect.bottom()),
                    ],
                    egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 100)),
                );
                painter.text(
                    egui::pos2(x + 2.0, ruler_rect.top() + 4.0),
                    egui::Align2::LEFT_TOP,
                    format!("{t:.1}s"),
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(140, 140, 160),
                );
                t += tick_step;
            }

            // Tracks
            for (row, track) in self.tracks.iter().enumerate() {
                let y_top = rect.top() + (row as f32 + 1.0) * row_h;
                let track_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left(), y_top),
                    egui::vec2(label_w + timeline_w, row_h),
                );
                let bg = if row % 2 == 0 {
                    Color32::from_rgb(28, 28, 36)
                } else {
                    Color32::from_rgb(32, 32, 40)
                };
                painter.rect_filled(track_rect, 0.0, bg);
                painter.text(
                    egui::pos2(rect.left() + 4.0, y_top + row_h * 0.5),
                    egui::Align2::LEFT_CENTER,
                    &track.name,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(200, 200, 220),
                );

                // Keyframes
                for kf in &track.keyframes {
                    let kx = ruler_rect.left() + kf.time * px_per_sec;
                    if kx < ruler_rect.left() || kx > ruler_rect.right() {
                        continue;
                    }
                    let ky = y_top + row_h * 0.5;
                    let kf_rect =
                        egui::Rect::from_center_size(egui::pos2(kx, ky), egui::vec2(10.0, 10.0));
                    painter.rect_filled(kf_rect, 2.0, Color32::from_rgb(220, 180, 60));
                    painter.rect_stroke(
                        kf_rect,
                        2.0,
                        egui::Stroke::new(1.0, Color32::WHITE),
                        egui::StrokeKind::Middle,
                    );
                }
            }

            // Playhead
            let phx = ruler_rect.left() + self.playhead * px_per_sec;
            if phx >= ruler_rect.left() && phx <= ruler_rect.right() {
                painter.line_segment(
                    [egui::pos2(phx, rect.top()), egui::pos2(phx, rect.bottom())],
                    egui::Stroke::new(2.0, Color32::from_rgb(240, 80, 80)),
                );
            }

            // Click to scrub
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let rel_x = pos.x - ruler_rect.left();
                    self.playhead = (rel_x / px_per_sec).clamp(0.0, self.duration);
                }
            }
        });

        // Request repaint while playing so the playhead advances.
        if self.playing {
            ui.ctx().request_repaint();
        }
    }
}
