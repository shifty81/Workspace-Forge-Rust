//! Animation Timeline & State Machine Editor panel for NovaForge Workspace.

use egui::Color32;
use novaforge_ui::{EditorPanel, PanelContext};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single keyframe on the timeline.
#[derive(Clone, Serialize, Deserialize)]
struct Keyframe {
    time: f32,
    label: String,
}

/// An animation track.
#[derive(Clone, Serialize, Deserialize)]
struct Track {
    name: String,
    keyframes: Vec<Keyframe>,
}

/// TOML-serialisable wrapper for the full animation clip.
#[derive(Serialize, Deserialize)]
struct AnimationFile {
    duration: f32,
    tracks: Vec<Track>,
}

/// Animation Editor panel.
pub struct AnimationEditor {
    tracks: Vec<Track>,
    /// Index of the currently selected track (for adding/deleting keyframes).
    selected_track: Option<usize>,
    /// Index within `selected_track` of the selected keyframe.
    selected_keyframe: Option<usize>,
    playhead: f32,
    duration: f32,
    playing: bool,
    zoom: f32,
    /// Monotonically increasing counter used to generate unique keyframe labels.
    keyframe_counter: u32,
    /// Text buffer used for the inline "new track name" input.
    new_track_name: String,
    /// Status message shown below the toolbar (save/load feedback).
    save_status: String,
    /// The (track_idx, keyframe_idx) that is currently being dragged, if any.
    dragging_kf: Option<(usize, usize)>,
}

impl Default for AnimationEditor {
    fn default() -> Self {
        Self {
            tracks: vec![
                Track {
                    name: "Root Bone".to_string(),
                    keyframes: vec![
                        Keyframe { time: 0.0, label: "A".to_string() },
                        Keyframe { time: 0.5, label: "B".to_string() },
                        Keyframe { time: 1.0, label: "C".to_string() },
                    ],
                },
                Track {
                    name: "Weapon".to_string(),
                    keyframes: vec![
                        Keyframe { time: 0.2, label: "A".to_string() },
                        Keyframe { time: 0.8, label: "B".to_string() },
                    ],
                },
                Track {
                    name: "FX".to_string(),
                    keyframes: vec![Keyframe { time: 0.6, label: "A".to_string() }],
                },
            ],
            selected_track: None,
            selected_keyframe: None,
            playhead: 0.0,
            duration: 2.0,
            playing: false,
            zoom: 1.0,
            keyframe_counter: 6, // 3 tracks × ~2 keyframes already in the defaults
            new_track_name: String::new(),
            save_status: String::new(),
            dragging_kf: None,
        }
    }
}

impl AnimationEditor {
    /// Canonical path for the animation file: `<asset_root>/animations/animation.toml`.
    fn anim_path(ctx: &PanelContext) -> Option<PathBuf> {
        ctx.asset_root
            .as_ref()
            .map(|r| r.join("animations").join("animation.toml"))
    }

    fn save_animation(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::anim_path(ctx) else {
            self.save_status = "No project loaded — cannot save animation.".to_string();
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.save_status = format!("Directory error: {e}");
                return;
            }
        }
        let file = AnimationFile {
            duration: self.duration,
            tracks: self.tracks.clone(),
        };
        match toml::to_string_pretty(&file) {
            Ok(content) => match std::fs::write(&path, content) {
                Ok(()) => self.save_status = format!("Saved → {}", path.display()),
                Err(e) => self.save_status = format!("Write error: {e}"),
            },
            Err(e) => self.save_status = format!("Serialise error: {e}"),
        }
    }

    fn load_animation(&mut self, ctx: &PanelContext) {
        let Some(path) = Self::anim_path(ctx) else {
            self.save_status = "No project loaded — cannot load animation.".to_string();
            return;
        };
        match std::fs::read_to_string(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.save_status = format!("File not found: {}", path.display());
            }
            Err(e) => self.save_status = format!("Read error: {e}"),
            Ok(content) => match toml::from_str::<AnimationFile>(&content) {
                Ok(file) => {
                    let n = file.tracks.len();
                    self.duration = file.duration;
                    self.tracks = file.tracks;
                    self.selected_track = None;
                    self.selected_keyframe = None;
                    self.playhead = 0.0;
                    self.save_status = format!("Loaded {n} tracks ← {}", path.display());
                }
                Err(e) => self.save_status = format!("Parse error: {e}"),
            },
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

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &PanelContext) {
        // Transport bar
        ui.horizontal(|ui| {
            if ui.button("⏮ Start").clicked() {
                self.playhead = 0.0;
            }
            let play_label = if self.playing { "⏸ Pause" } else { "▶ Play" };
            if ui.button(play_label).clicked() {
                self.playing = !self.playing;
            }
            if ui.button("⏹ Stop").clicked() {
                self.playing = false;
                self.playhead = 0.0;
            }
            ui.separator();
            ui.label(format!("Time: {:.2} / {:.2} s", self.playhead, self.duration));
            ui.separator();
            ui.label("Zoom:");
            if ui.button("＋").clicked() {
                self.zoom = (self.zoom * 1.2).min(8.0);
            }
            if ui.button("−").clicked() {
                self.zoom = (self.zoom / 1.2).max(0.25);
            }
        });

        // Keyframe & track controls
        ui.horizontal(|ui| {
            let track_sel = self.selected_track.is_some();
            let kf_sel = self.selected_keyframe.is_some();

            if ui
                .add_enabled(track_sel, egui::Button::new("＋ Keyframe"))
                .on_hover_text("Add keyframe at playhead on selected track")
                .clicked()
            {
                if let Some(ti) = self.selected_track {
                    if let Some(track) = self.tracks.get_mut(ti) {
                        let t = self.playhead;
                        // Avoid duplicate keyframe at same time (within 1 ms).
                        if !track.keyframes.iter().any(|k| (k.time - t).abs() < 0.001) {
                            self.keyframe_counter += 1;
                            let label =
                                (b'A' + (self.keyframe_counter % 26) as u8) as char;
                            track.keyframes.push(Keyframe {
                                time: t,
                                label: label.to_string(),
                            });
                            track
                                .keyframes
                                .sort_by(|a, b| a.time.total_cmp(&b.time));
                            self.selected_keyframe = track
                                .keyframes
                                .iter()
                                .position(|k| (k.time - t).abs() < 0.001);
                        }
                    }
                }
            }
            if ui
                .add_enabled(track_sel && kf_sel, egui::Button::new("🗑 Keyframe"))
                .on_hover_text("Delete selected keyframe")
                .clicked()
            {
                if let (Some(ti), Some(ki)) = (self.selected_track, self.selected_keyframe) {
                    if let Some(track) = self.tracks.get_mut(ti) {
                        if ki < track.keyframes.len() {
                            track.keyframes.remove(ki);
                            self.selected_keyframe = None;
                        }
                    }
                }
            }

            ui.separator();

            // Track management
            ui.label("Track:");
            ui.add(
                egui::TextEdit::singleline(&mut self.new_track_name)
                    .hint_text("name…")
                    .desired_width(90.0),
            );
            let can_add_track = !self.new_track_name.trim().is_empty();
            if ui
                .add_enabled(can_add_track, egui::Button::new("＋ Track"))
                .on_hover_text("Add a new named track")
                .clicked()
            {
                let name = self.new_track_name.trim().to_string();
                // Deduplicate: append a numeric suffix if the name already exists.
                let base = name.clone();
                let mut final_name = base.clone();
                let mut suffix = 2u32;
                while self.tracks.iter().any(|t| t.name == final_name) {
                    final_name = format!("{base} {suffix}");
                    suffix += 1;
                }
                self.tracks.push(Track { name: final_name, keyframes: Vec::new() });
                self.new_track_name.clear();
                self.selected_track = Some(self.tracks.len() - 1);
                self.selected_keyframe = None;
            }
            if ui
                .add_enabled(track_sel, egui::Button::new("🗑 Track"))
                .on_hover_text("Delete selected track and all its keyframes")
                .clicked()
            {
                if let Some(ti) = self.selected_track {
                    if ti < self.tracks.len() {
                        self.tracks.remove(ti);
                        self.selected_track = if self.tracks.is_empty() {
                            None
                        } else {
                            Some(ti.min(self.tracks.len() - 1))
                        };
                        self.selected_keyframe = None;
                    }
                }
            }

            ui.separator();
            if ui
                .button("💾 Save")
                .on_hover_text("Save animation to <asset_root>/animations/animation.toml")
                .clicked()
            {
                self.save_animation(ctx);
            }
            if ui
                .button("📂 Load")
                .on_hover_text("Load animation from <asset_root>/animations/animation.toml")
                .clicked()
            {
                self.load_animation(ctx);
            }
        });

        // Status line
        if !self.save_status.is_empty() {
            ui.label(
                egui::RichText::new(&self.save_status)
                    .size(11.0)
                    .color(Color32::from_rgb(160, 200, 160)),
            );
        }

        // Track info hint
        if let Some(ti) = self.selected_track {
            if let Some(track) = self.tracks.get(ti) {
                ui.label(
                    egui::RichText::new(format!(
                        "Track: {}  ({} keyframes)",
                        track.name,
                        track.keyframes.len()
                    ))
                    .size(11.0)
                    .color(Color32::from_rgb(140, 180, 240)),
                );
            }
        }

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
            let tick_step = if self.zoom > 2.0 { 0.1 } else if self.zoom > 0.5 { 0.5 } else { 1.0 };
            let mut t = 0.0_f32;
            while t <= self.duration {
                let x = ruler_rect.left() + t * px_per_sec;
                if x > ruler_rect.right() {
                    break;
                }
                painter.line_segment(
                    [egui::pos2(x, ruler_rect.top()), egui::pos2(x, ruler_rect.bottom())],
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
            let mut new_selected_track = self.selected_track;
            let mut new_selected_kf = self.selected_keyframe;
            let mut new_dragging_kf = self.dragging_kf;
            // Pending keyframe time update (track_idx, kf_idx, new_time).
            let mut kf_time_update: Option<(usize, usize, f32)> = None;
            let mut kf_drag_stopped = false;

            for (row, track) in self.tracks.iter().enumerate() {
                let y_top = rect.top() + (row as f32 + 1.0) * row_h;
                let track_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left(), y_top),
                    egui::vec2(label_w + timeline_w, row_h),
                );
                let track_selected = self.selected_track == Some(row);
                let bg = if track_selected {
                    Color32::from_rgb(40, 40, 60)
                } else if row % 2 == 0 {
                    Color32::from_rgb(28, 28, 36)
                } else {
                    Color32::from_rgb(32, 32, 40)
                };
                painter.rect_filled(track_rect, 0.0, bg);

                // Track label — click to select the track
                let label_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left(), y_top),
                    egui::vec2(label_w, row_h),
                );
                let label_response = ui.interact(label_rect, ui.id().with(("track_label", row)), egui::Sense::click());
                if label_response.clicked() {
                    new_selected_track = Some(row);
                    new_selected_kf = None;
                }
                painter.text(
                    egui::pos2(rect.left() + 4.0, y_top + row_h * 0.5),
                    egui::Align2::LEFT_CENTER,
                    &track.name,
                    egui::FontId::proportional(11.0),
                    if track_selected {
                        Color32::from_rgb(220, 220, 255)
                    } else {
                        Color32::from_rgb(200, 200, 220)
                    },
                );

                // Keyframes
                let is_kf_dragging = self.dragging_kf.map(|(ti, _)| ti) == Some(row);
                for (ki, kf) in track.keyframes.iter().enumerate() {
                    let kx = ruler_rect.left() + kf.time * px_per_sec;
                    if kx < ruler_rect.left() || kx > ruler_rect.right() {
                        continue;
                    }
                    let ky = y_top + row_h * 0.5;
                    let kf_rect = egui::Rect::from_center_size(egui::pos2(kx, ky), egui::vec2(10.0, 10.0));
                    let kf_selected = self.selected_track == Some(row) && self.selected_keyframe == Some(ki);
                    let kf_being_dragged = self.dragging_kf == Some((row, ki));
                    let kf_color = if kf_selected || kf_being_dragged {
                        Color32::from_rgb(255, 220, 60)
                    } else {
                        Color32::from_rgb(200, 160, 40)
                    };
                    painter.rect_filled(kf_rect, 2.0, kf_color);
                    painter.rect_stroke(
                        kf_rect,
                        2.0,
                        egui::Stroke::new(if kf_selected || kf_being_dragged { 2.0 } else { 1.0 }, Color32::WHITE),
                        egui::StrokeKind::Middle,
                    );
                    // Show a drag-cursor hint when hovering over a keyframe.
                    if kf_being_dragged {
                        painter.rect_filled(
                            egui::Rect::from_center_size(egui::pos2(kx, ky), egui::vec2(2.0, row_h)),
                            0.0,
                            Color32::from_rgba_premultiplied(255, 220, 60, 80),
                        );
                    }
                    // Draw the keyframe label (first character) inside the diamond.
                    if let Some(ch) = kf.label.chars().next() {
                        painter.text(
                            egui::pos2(kx, y_top + row_h * 0.5),
                            egui::Align2::CENTER_CENTER,
                            ch.to_string(),
                            egui::FontId::proportional(7.0),
                            Color32::from_rgb(30, 20, 0),
                        );
                    }
                    // Interact — click to select, drag to reposition.
                    let kf_response = ui.interact(
                        kf_rect,
                        ui.id().with(("kf", row, ki)),
                        egui::Sense::click_and_drag(),
                    );
                    if kf_response.clicked() && !is_kf_dragging {
                        new_selected_track = Some(row);
                        new_selected_kf = Some(ki);
                    }
                    if kf_response.drag_started() {
                        new_dragging_kf = Some((row, ki));
                        new_selected_track = Some(row);
                        new_selected_kf = Some(ki);
                    }
                    if kf_response.dragged() {
                        let dt = kf_response.drag_delta().x / px_per_sec;
                        let new_time = (kf.time + dt).clamp(0.0, self.duration);
                        kf_time_update = Some((row, ki, new_time));
                    }
                    if kf_response.drag_stopped() {
                        kf_drag_stopped = true;
                    }
                }
            }

            // Apply collected state updates (after the immutable borrow of self.tracks ends).
            self.selected_track = new_selected_track;
            self.selected_keyframe = new_selected_kf;
            self.dragging_kf = new_dragging_kf;

            if let Some((ti, ki, new_time)) = kf_time_update {
                if let Some(track) = self.tracks.get_mut(ti) {
                    if let Some(kf) = track.keyframes.get_mut(ki) {
                        kf.time = new_time;
                    }
                }
            }
            if kf_drag_stopped {
                if let Some((ti, _)) = self.dragging_kf {
                    if let Some(track) = self.tracks.get_mut(ti) {
                        track.keyframes.sort_by(|a, b| a.time.total_cmp(&b.time));
                    }
                }
                self.dragging_kf = None;
                // Index is invalid after re-sorting; user can re-click.
                self.selected_keyframe = None;
            }

            // Playhead
            let phx = ruler_rect.left() + self.playhead * px_per_sec;
            if phx >= ruler_rect.left() && phx <= ruler_rect.right() {
                painter.line_segment(
                    [egui::pos2(phx, rect.top()), egui::pos2(phx, rect.bottom())],
                    egui::Stroke::new(2.0, Color32::from_rgb(240, 80, 80)),
                );
            }

            // Click on ruler/timeline background to scrub playhead
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if pos.y < rect.top() + row_h {
                        // Clicked in the ruler row → scrub
                        let rel_x = pos.x - ruler_rect.left();
                        self.playhead = (rel_x / px_per_sec).clamp(0.0, self.duration);
                    }
                }
            }
        });

        // Request repaint while playing so the playhead advances.
        if self.playing {
            ui.ctx().request_repaint();
        }
    }
}
