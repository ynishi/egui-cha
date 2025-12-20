//! Transport controls for audio/video playback
//!
//! Provides Play/Stop/Record buttons and BPM display for DAW-style applications.
//!
//! # Example
//! ```ignore
//! TransportBar::new()
//!     .playing(model.is_playing)
//!     .recording(model.is_recording)
//!     .bpm(model.bpm)
//!     .show_with(ctx, |event| match event {
//!         TransportEvent::Play => Msg::Play,
//!         TransportEvent::Stop => Msg::Stop,
//!         TransportEvent::Record => Msg::ToggleRecord,
//!         TransportEvent::BpmChange(bpm) => Msg::SetBpm(bpm),
//!     });
//! ```

use crate::Theme;
use egui::{CornerRadius, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Transport control events
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransportEvent {
    /// Play button pressed
    Play,
    /// Stop button pressed
    Stop,
    /// Record button toggled
    Record,
    /// BPM changed (via drag or input)
    BpmChange(f32),
}

/// Transport bar with playback controls
pub struct TransportBar {
    playing: bool,
    recording: bool,
    bpm: f32,
    show_record: bool,
    show_bpm: bool,
    compact: bool,
}

impl Default for TransportBar {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportBar {
    /// Create a new transport bar
    pub fn new() -> Self {
        Self {
            playing: false,
            recording: false,
            bpm: 120.0,
            show_record: true,
            show_bpm: true,
            compact: false,
        }
    }

    /// Set playing state
    pub fn playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    /// Set recording state
    pub fn recording(mut self, recording: bool) -> Self {
        self.recording = recording;
        self
    }

    /// Set BPM value
    pub fn bpm(mut self, bpm: f32) -> Self {
        self.bpm = bpm;
        self
    }

    /// Show/hide record button
    pub fn show_record(mut self, show: bool) -> Self {
        self.show_record = show;
        self
    }

    /// Show/hide BPM display
    pub fn show_bpm(mut self, show: bool) -> Self {
        self.show_bpm = show;
        self
    }

    /// Compact mode (smaller buttons)
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// TEA-style: Show transport bar and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(TransportEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show transport bar, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<TransportEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<TransportEvent> {
        let theme = Theme::current(ui.ctx());
        let button_size = if self.compact { 28.0 } else { 36.0 };
        let spacing = theme.spacing_xs;

        let mut event = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;

            // Stop button (square)
            if self.render_stop_button(ui, button_size, &theme) {
                event = Some(TransportEvent::Stop);
            }

            // Play button (triangle)
            if self.render_play_button(ui, button_size, &theme) {
                event = Some(TransportEvent::Play);
            }

            // Record button (circle)
            if self.show_record {
                if self.render_record_button(ui, button_size, &theme) {
                    event = Some(TransportEvent::Record);
                }
            }

            // BPM display/control
            if self.show_bpm {
                ui.add_space(spacing * 2.0);
                if let Some(new_bpm) = self.render_bpm(ui, &theme) {
                    event = Some(TransportEvent::BpmChange(new_bpm));
                }
            }
        });

        event
    }

    fn render_stop_button(&self, ui: &mut Ui, size: f32, theme: &Theme) -> bool {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let bg_color = if !self.playing {
                theme.primary
            } else {
                visuals.bg_fill
            };

            // Background
            ui.painter().rect_filled(rect, CornerRadius::same(4), bg_color);

            // Square icon
            let icon_size = size * 0.35;
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
            let icon_color = if !self.playing {
                theme.primary_text
            } else {
                visuals.fg_stroke.color
            };
            ui.painter().rect_filled(icon_rect, CornerRadius::ZERO, icon_color);
        }

        response.clicked()
    }

    fn render_play_button(&self, ui: &mut Ui, size: f32, theme: &Theme) -> bool {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let bg_color = if self.playing {
                theme.state_success
            } else {
                visuals.bg_fill
            };

            // Background
            ui.painter().rect_filled(rect, CornerRadius::same(4), bg_color);

            // Triangle icon (play)
            let icon_size = size * 0.4;
            let center = rect.center();
            let icon_color = if self.playing {
                theme.state_success_text
            } else {
                visuals.fg_stroke.color
            };

            let points = vec![
                egui::pos2(center.x - icon_size * 0.4, center.y - icon_size * 0.5),
                egui::pos2(center.x + icon_size * 0.5, center.y),
                egui::pos2(center.x - icon_size * 0.4, center.y + icon_size * 0.5),
            ];
            ui.painter().add(egui::Shape::convex_polygon(
                points,
                icon_color,
                Stroke::NONE,
            ));
        }

        response.clicked()
    }

    fn render_record_button(&self, ui: &mut Ui, size: f32, theme: &Theme) -> bool {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let bg_color = if self.recording {
                theme.state_danger
            } else {
                visuals.bg_fill
            };

            // Background
            ui.painter().rect_filled(rect, CornerRadius::same(4), bg_color);

            // Circle icon (record)
            let icon_radius = size * 0.2;
            let icon_color = if self.recording {
                theme.state_danger_text
            } else {
                theme.state_danger
            };
            ui.painter().circle_filled(rect.center(), icon_radius, icon_color);
        }

        response.clicked()
    }

    fn render_bpm(&self, ui: &mut Ui, theme: &Theme) -> Option<f32> {
        let mut new_bpm = None;

        ui.horizontal(|ui| {
            // BPM label
            ui.label(
                egui::RichText::new("BPM")
                    .size(theme.font_size_sm)
                    .color(theme.text_secondary),
            );

            // BPM value (draggable)
            let response = ui.add(
                egui::DragValue::new(&mut self.bpm.clone())
                    .range(20.0..=300.0)
                    .speed(0.5)
                    .fixed_decimals(1)
                    .custom_formatter(|v, _| format!("{:.1}", v)),
            );

            if response.changed() {
                // Get the actual value from the drag
                if let Some(bpm) = response.drag_delta().y.abs().gt(&0.0).then(|| {
                    (self.bpm - response.drag_delta().y * 0.5).clamp(20.0, 300.0)
                }) {
                    new_bpm = Some(bpm);
                }
            }
        });

        new_bpm
    }
}

/// Compact beat indicator showing current beat position
pub struct BeatIndicator {
    beats: usize,
    current: usize,
    size: f32,
}

impl BeatIndicator {
    /// Create a new beat indicator
    pub fn new(beats: usize) -> Self {
        Self {
            beats,
            current: 0,
            size: 16.0,
        }
    }

    /// Set current beat (0-indexed)
    pub fn current_beat(mut self, beat: usize) -> Self {
        self.current = beat;
        self
    }

    /// Set indicator size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Show the beat indicator
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let spacing = theme.spacing_xs * 0.5;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;

            for i in 0..self.beats {
                let (rect, _response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());

                if ui.is_rect_visible(rect) {
                    let is_current = i == self.current;
                    let is_downbeat = i == 0;

                    let color = if is_current {
                        if is_downbeat {
                            theme.state_warning // Downbeat accent
                        } else {
                            theme.primary
                        }
                    } else {
                        theme.bg_tertiary
                    };

                    let radius = if is_current {
                        self.size * 0.45
                    } else {
                        self.size * 0.35
                    };

                    ui.painter().circle_filled(rect.center(), radius, color);

                    // Border for inactive
                    if !is_current {
                        ui.painter().circle_stroke(
                            rect.center(),
                            radius,
                            Stroke::new(1.0, theme.border),
                        );
                    }
                }
            }
        });
    }
}
