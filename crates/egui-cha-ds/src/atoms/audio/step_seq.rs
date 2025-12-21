//! Step Sequencer - TR-808/909 style grid sequencer
//!
//! A grid-based step sequencer for drum machines and melodic patterns.
//!
//! # Example
//! ```ignore
//! StepSeq::new(4, 16)  // 4 tracks x 16 steps
//!     .track_labels(&["Kick", "Snare", "HiHat", "Clap"])
//!     .playhead(model.current_step)
//!     .accent_every(4)  // Mark every 4th step as accent
//!     .show_with(ctx, &model.pattern, |event| match event {
//!         StepEvent::Toggle { track, step } => Msg::ToggleStep(track, step),
//!         StepEvent::SetValue { track, step, value } => Msg::SetStepValue(track, step, value),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Step value representing the state of a single step
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StepValue {
    /// Step is off
    #[default]
    Off,
    /// Step is on with normal velocity
    On,
    /// Step is on with accent (high velocity)
    Accent,
    /// Step is on with specific velocity (0.0 - 1.0)
    Velocity(f32),
}

impl StepValue {
    /// Check if step is active
    pub fn is_on(&self) -> bool {
        !matches!(self, StepValue::Off)
    }

    /// Toggle between Off and On
    pub fn toggle(self) -> Self {
        match self {
            StepValue::Off => StepValue::On,
            _ => StepValue::Off,
        }
    }

    /// Cycle through Off -> On -> Accent -> Off
    pub fn cycle(self) -> Self {
        match self {
            StepValue::Off => StepValue::On,
            StepValue::On => StepValue::Accent,
            StepValue::Accent => StepValue::Off,
            StepValue::Velocity(_) => StepValue::Off,
        }
    }
}

/// Events from step sequencer interactions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepEvent {
    /// Toggle step on/off
    Toggle { track: usize, step: usize },
    /// Set specific value for a step
    SetValue {
        track: usize,
        step: usize,
        value: StepValue,
    },
}

/// Pattern data type (tracks x steps)
pub type Pattern = Vec<Vec<StepValue>>;

/// TR-808/909 style step sequencer grid
pub struct StepSeq<'a> {
    tracks: usize,
    steps: usize,
    track_labels: Option<&'a [&'a str]>,
    playhead: Option<usize>,
    accent_every: Option<usize>,
    step_size: Vec2,
    spacing: f32,
    label_width: f32,
    show_beat_numbers: bool,
    colors: Option<&'a [Color32]>,
}

impl<'a> StepSeq<'a> {
    /// Create a new step sequencer
    pub fn new(tracks: usize, steps: usize) -> Self {
        Self {
            tracks: tracks.max(1),
            steps: steps.max(1),
            track_labels: None,
            playhead: None,
            accent_every: None,
            step_size: Vec2::new(24.0, 20.0),
            spacing: 2.0,
            label_width: 60.0,
            show_beat_numbers: true,
            colors: None,
        }
    }

    /// Set track labels (displayed on left side)
    pub fn track_labels(mut self, labels: &'a [&'a str]) -> Self {
        self.track_labels = Some(labels);
        self
    }

    /// Set current playhead position (0-indexed step)
    pub fn playhead(mut self, step: Option<usize>) -> Self {
        self.playhead = step;
        self
    }

    /// Mark every Nth step as an accent/downbeat
    pub fn accent_every(mut self, n: usize) -> Self {
        self.accent_every = Some(n);
        self
    }

    /// Set step cell size
    pub fn step_size(mut self, width: f32, height: f32) -> Self {
        self.step_size = Vec2::new(width, height);
        self
    }

    /// Set spacing between cells
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set label column width
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = width;
        self
    }

    /// Show beat numbers above grid
    pub fn show_beat_numbers(mut self, show: bool) -> Self {
        self.show_beat_numbers = show;
        self
    }

    /// Set custom colors per track
    pub fn colors(mut self, colors: &'a [Color32]) -> Self {
        self.colors = Some(colors);
        self
    }

    /// TEA-style: Show sequencer and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        pattern: &Pattern,
        on_event: impl Fn(StepEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui, pattern) {
            ctx.emit(on_event(event));
        }
    }

    /// Show sequencer, returns event if any
    pub fn show(self, ui: &mut Ui, pattern: &Pattern) -> Option<StepEvent> {
        self.render(ui, pattern)
    }

    fn render(self, ui: &mut Ui, pattern: &Pattern) -> Option<StepEvent> {
        let theme = Theme::current(ui.ctx());
        let time = ui.input(|i| i.time) as f32;
        let mut event = None;

        // Calculate dimensions
        let header_height = if self.show_beat_numbers { 16.0 } else { 0.0 };
        let grid_width = self.steps as f32 * self.step_size.x
            + (self.steps.saturating_sub(1)) as f32 * self.spacing;
        let grid_height = self.tracks as f32 * self.step_size.y
            + (self.tracks.saturating_sub(1)) as f32 * self.spacing;

        let total_width = self.label_width + theme.spacing_sm + grid_width;
        let total_height = header_height + grid_height;

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let grid_start_x = rect.min.x + self.label_width + theme.spacing_sm;
        let grid_start_y = rect.min.y + header_height;

        // First pass: collect step info and handle interactions
        struct StepInfo {
            rect: Rect,
            track: usize,
            step: usize,
            value: StepValue,
            is_playhead: bool,
            is_accent_step: bool,
            hovered: bool,
            track_color: Color32,
        }

        let mut steps_info: Vec<StepInfo> = Vec::with_capacity(self.tracks * self.steps);

        for track in 0..self.tracks {
            let track_y = grid_start_y + track as f32 * (self.step_size.y + self.spacing);
            let track_color = self
                .colors
                .and_then(|c| c.get(track))
                .copied()
                .unwrap_or(theme.primary);

            for step in 0..self.steps {
                let step_x = grid_start_x + step as f32 * (self.step_size.x + self.spacing);
                let step_rect = Rect::from_min_size(egui::pos2(step_x, track_y), self.step_size);

                let step_value = pattern
                    .get(track)
                    .and_then(|t| t.get(step))
                    .copied()
                    .unwrap_or(StepValue::Off);

                let is_playhead = self.playhead == Some(step);
                let is_accent_step = self.accent_every.map(|n| step % n == 0).unwrap_or(false);

                // Allocate interactive area
                let response = ui.allocate_rect(step_rect, Sense::click());

                // Handle interactions
                if response.clicked() {
                    if ui.input(|i| i.modifiers.shift) {
                        event = Some(StepEvent::SetValue {
                            track,
                            step,
                            value: step_value.cycle(),
                        });
                    } else {
                        event = Some(StepEvent::Toggle { track, step });
                    }
                }

                steps_info.push(StepInfo {
                    rect: step_rect,
                    track,
                    step,
                    value: step_value,
                    is_playhead,
                    is_accent_step,
                    hovered: response.hovered(),
                    track_color,
                });
            }
        }

        // Second pass: draw everything
        let painter = ui.painter();

        // Draw beat numbers
        if self.show_beat_numbers {
            for step in 0..self.steps {
                let x = grid_start_x + step as f32 * (self.step_size.x + self.spacing);
                let is_accent = self.accent_every.map(|n| step % n == 0).unwrap_or(false);

                if is_accent || step % 4 == 0 {
                    let beat_num = (step / 4) + 1;
                    painter.text(
                        egui::pos2(x + self.step_size.x / 2.0, rect.min.y + header_height / 2.0),
                        egui::Align2::CENTER_CENTER,
                        format!("{}", beat_num),
                        egui::FontId::proportional(theme.font_size_xs * 0.9),
                        if is_accent {
                            theme.text_primary
                        } else {
                            theme.text_muted
                        },
                    );
                }
            }
        }

        // Draw track labels
        for track in 0..self.tracks {
            let track_y = grid_start_y + track as f32 * (self.step_size.y + self.spacing);
            if let Some(labels) = self.track_labels {
                if let Some(label) = labels.get(track) {
                    let label_rect = Rect::from_min_size(
                        egui::pos2(rect.min.x, track_y),
                        Vec2::new(self.label_width, self.step_size.y),
                    );
                    painter.text(
                        egui::pos2(label_rect.right() - 4.0, label_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        *label,
                        egui::FontId::proportional(theme.font_size_xs),
                        theme.text_secondary,
                    );
                }
            }
        }

        // Draw steps
        for step_info in &steps_info {
            let (bg_color, border_color) = match step_info.value {
                StepValue::Off => {
                    let bg = if step_info.is_accent_step {
                        Color32::from_rgba_unmultiplied(
                            step_info.track_color.r(),
                            step_info.track_color.g(),
                            step_info.track_color.b(),
                            20,
                        )
                    } else if step_info.hovered {
                        theme.bg_tertiary
                    } else {
                        theme.bg_secondary
                    };
                    (bg, theme.border)
                }
                StepValue::On => {
                    let bg = Color32::from_rgba_unmultiplied(
                        step_info.track_color.r(),
                        step_info.track_color.g(),
                        step_info.track_color.b(),
                        180,
                    );
                    (bg, step_info.track_color)
                }
                StepValue::Accent => (step_info.track_color, theme.state_warning),
                StepValue::Velocity(v) => {
                    let bg = Color32::from_rgba_unmultiplied(
                        step_info.track_color.r(),
                        step_info.track_color.g(),
                        step_info.track_color.b(),
                        (v * 200.0 + 55.0) as u8,
                    );
                    (bg, step_info.track_color)
                }
            };

            // Draw step background
            painter.rect_filled(step_info.rect, theme.radius_sm * 0.5, bg_color);

            // Draw border
            painter.rect_stroke(
                step_info.rect,
                theme.radius_sm * 0.5,
                Stroke::new(theme.border_width, border_color),
                egui::StrokeKind::Inside,
            );

            // Playhead overlay
            if step_info.is_playhead {
                let playhead_alpha = ((time * 4.0).sin() * 0.2 + 0.4).clamp(0.2, 0.6);
                painter.rect_filled(
                    step_info.rect,
                    theme.radius_sm * 0.5,
                    Color32::from_rgba_unmultiplied(255, 255, 255, (playhead_alpha * 255.0) as u8),
                );
            }

            // Accent indicator (small triangle)
            if matches!(step_info.value, StepValue::Accent) {
                let indicator_size = 4.0;
                let center = step_info.rect.right_top()
                    + Vec2::new(-indicator_size - 1.0, indicator_size + 1.0);
                let points = vec![
                    egui::pos2(center.x, center.y - indicator_size),
                    egui::pos2(center.x - indicator_size, center.y),
                    egui::pos2(center.x + indicator_size, center.y),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points,
                    theme.state_warning,
                    Stroke::NONE,
                ));
            }
        }

        // Draw playhead line
        if let Some(step) = self.playhead {
            let line_x = grid_start_x
                + step as f32 * (self.step_size.x + self.spacing)
                + self.step_size.x / 2.0;
            painter.line_segment(
                [
                    egui::pos2(line_x, grid_start_y - 2.0),
                    egui::pos2(line_x, grid_start_y + grid_height + 2.0),
                ],
                Stroke::new(2.0, theme.state_success),
            );
        }

        // Request repaint if playing (for animation)
        if self.playhead.is_some() {
            ui.ctx().request_repaint();
        }

        event
    }
}
