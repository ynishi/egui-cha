//! BeatSync atom - Tap tempo and beat synchronization for VJ applications
//!
//! A component for BPM input via tap tempo and visual beat sync indicator.
//! Supports manual BPM entry, tap detection, and beat phase visualization.

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Beat division for sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BeatDivision {
    #[default]
    Quarter,
    Eighth,
    Sixteenth,
    Half,
    Whole,
    Triplet,
}

impl BeatDivision {
    pub fn label(&self) -> &'static str {
        match self {
            BeatDivision::Quarter => "1/4",
            BeatDivision::Eighth => "1/8",
            BeatDivision::Sixteenth => "1/16",
            BeatDivision::Half => "1/2",
            BeatDivision::Whole => "1",
            BeatDivision::Triplet => "1/3",
        }
    }

    pub fn multiplier(&self) -> f32 {
        match self {
            BeatDivision::Quarter => 1.0,
            BeatDivision::Eighth => 0.5,
            BeatDivision::Sixteenth => 0.25,
            BeatDivision::Half => 2.0,
            BeatDivision::Whole => 4.0,
            BeatDivision::Triplet => 1.0 / 3.0,
        }
    }

    pub fn all() -> &'static [BeatDivision] {
        &[
            BeatDivision::Whole,
            BeatDivision::Half,
            BeatDivision::Quarter,
            BeatDivision::Eighth,
            BeatDivision::Sixteenth,
            BeatDivision::Triplet,
        ]
    }
}

/// Sync state data
#[derive(Debug, Clone)]
pub struct SyncState {
    pub bpm: f32,
    pub phase: f32,
    pub beat_count: u32,
    pub division: BeatDivision,
    pub is_synced: bool,
    pub tap_times: Vec<f64>,
}

impl SyncState {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm: bpm.clamp(20.0, 300.0),
            phase: 0.0,
            beat_count: 0,
            division: BeatDivision::Quarter,
            is_synced: false,
            tap_times: Vec::new(),
        }
    }

    pub fn beat_duration_ms(&self) -> f32 {
        60_000.0 / self.bpm * self.division.multiplier()
    }

    pub fn update_phase(&mut self, delta_ms: f32) {
        let beat_ms = self.beat_duration_ms();
        self.phase += delta_ms / beat_ms;
        while self.phase >= 1.0 {
            self.phase -= 1.0;
            self.beat_count = self.beat_count.wrapping_add(1);
        }
    }

    pub fn add_tap(&mut self, time: f64) {
        self.tap_times.push(time);
        if self.tap_times.len() > 8 {
            self.tap_times.remove(0);
        }
        if self.tap_times.len() >= 2 {
            self.calculate_bpm_from_taps();
        }
    }

    fn calculate_bpm_from_taps(&mut self) {
        if self.tap_times.len() < 2 {
            return;
        }
        let mut intervals: Vec<f64> = Vec::new();
        for i in 1..self.tap_times.len() {
            intervals.push(self.tap_times[i] - self.tap_times[i - 1]);
        }
        let avg_interval = intervals.iter().sum::<f64>() / intervals.len() as f64;
        if avg_interval > 0.0 {
            self.bpm = (60.0 / avg_interval as f32).clamp(20.0, 300.0);
            self.is_synced = true;
        }
    }

    pub fn reset_taps(&mut self) {
        self.tap_times.clear();
        self.is_synced = false;
    }

    pub fn resync(&mut self) {
        self.phase = 0.0;
        self.beat_count = 0;
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new(120.0)
    }
}

/// Events emitted by BeatSync
#[derive(Debug, Clone)]
pub enum BeatSyncEvent {
    Tap(f64),
    SetBpm(f32),
    SetDivision(BeatDivision),
    Resync,
    ResetTaps,
    NudgePhase(f32),
}

/// Beat sync widget
pub struct BeatSync<'a> {
    state: &'a SyncState,
    size: Vec2,
    show_tap_button: bool,
    show_division: bool,
    show_phase_indicator: bool,
    show_bpm_adjust: bool,
    compact: bool,
}

impl<'a> BeatSync<'a> {
    pub fn new(state: &'a SyncState) -> Self {
        Self {
            state,
            size: Vec2::new(200.0, 80.0),
            show_tap_button: true,
            show_division: true,
            show_phase_indicator: true,
            show_bpm_adjust: true,
            compact: false,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        if compact {
            self.size = Vec2::new(120.0, 40.0);
        }
        self
    }

    pub fn show_tap_button(mut self, show: bool) -> Self {
        self.show_tap_button = show;
        self
    }

    pub fn show_division(mut self, show: bool) -> Self {
        self.show_division = show;
        self
    }

    pub fn show_phase_indicator(mut self, show: bool) -> Self {
        self.show_phase_indicator = show;
        self
    }

    pub fn show_bpm_adjust(mut self, show: bool) -> Self {
        self.show_bpm_adjust = show;
        self
    }

    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(BeatSyncEvent) -> Msg,
    ) {
        if let Some(e) = self.show_internal(ctx.ui) {
            ctx.emit(on_event(e));
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<BeatSyncEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<BeatSyncEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<BeatSyncEvent> = None;

        let (rect, _response) = ui.allocate_exact_size(self.size, Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        if self.compact {
            return self.show_compact(ui, rect, &theme);
        }

        // Layout calculations
        let padding = theme.spacing_sm;
        let inner_rect = rect.shrink(padding);

        // Collect interactions first
        struct Interactions {
            tap_clicked: bool,
            resync_clicked: bool,
            minus_clicked: bool,
            plus_clicked: bool,
            division_clicked: Option<BeatDivision>,
            nudge_left: bool,
            nudge_right: bool,
        }

        let mut interactions = Interactions {
            tap_clicked: false,
            resync_clicked: false,
            minus_clicked: false,
            plus_clicked: false,
            division_clicked: None,
            nudge_left: false,
            nudge_right: false,
        };

        // Tap button
        let tap_width = 50.0;
        let tap_rect = Rect::from_min_size(
            inner_rect.min,
            Vec2::new(tap_width, inner_rect.height()),
        );
        if self.show_tap_button {
            let resp = ui.allocate_rect(tap_rect, Sense::click());
            interactions.tap_clicked = resp.clicked();
        }

        // BPM display area (center)
        let bpm_x = inner_rect.min.x + tap_width + padding;
        let bpm_width = inner_rect.width() - tap_width * 2.0 - padding * 2.0;
        let bpm_rect = Rect::from_min_size(
            Pos2::new(bpm_x, inner_rect.min.y),
            Vec2::new(bpm_width, inner_rect.height() * 0.6),
        );

        // BPM +/- buttons
        if self.show_bpm_adjust {
            let btn_size = theme.spacing_lg;
            let minus_rect = Rect::from_min_size(
                Pos2::new(bpm_rect.min.x, bpm_rect.center().y - btn_size / 2.0),
                Vec2::splat(btn_size),
            );
            let plus_rect = Rect::from_min_size(
                Pos2::new(bpm_rect.max.x - btn_size, bpm_rect.center().y - btn_size / 2.0),
                Vec2::splat(btn_size),
            );

            let minus_resp = ui.allocate_rect(minus_rect, Sense::click());
            let plus_resp = ui.allocate_rect(plus_rect, Sense::click());
            interactions.minus_clicked = minus_resp.clicked();
            interactions.plus_clicked = plus_resp.clicked();
        }

        // Phase indicator / nudge buttons
        let phase_rect = Rect::from_min_size(
            Pos2::new(bpm_x, inner_rect.min.y + inner_rect.height() * 0.6 + padding),
            Vec2::new(bpm_width, inner_rect.height() * 0.4 - padding),
        );

        if self.show_phase_indicator {
            let nudge_width = theme.spacing_md;
            let left_nudge = Rect::from_min_size(phase_rect.min, Vec2::new(nudge_width, phase_rect.height()));
            let right_nudge = Rect::from_min_size(
                Pos2::new(phase_rect.max.x - nudge_width, phase_rect.min.y),
                Vec2::new(nudge_width, phase_rect.height()),
            );

            let left_resp = ui.allocate_rect(left_nudge, Sense::click());
            let right_resp = ui.allocate_rect(right_nudge, Sense::click());
            interactions.nudge_left = left_resp.clicked();
            interactions.nudge_right = right_resp.clicked();
        }

        // Resync button (right side)
        let resync_rect = Rect::from_min_size(
            Pos2::new(inner_rect.max.x - tap_width, inner_rect.min.y),
            Vec2::new(tap_width, inner_rect.height() * 0.5),
        );
        let resync_resp = ui.allocate_rect(resync_rect, Sense::click());
        interactions.resync_clicked = resync_resp.clicked();

        // Division buttons
        if self.show_division {
            let div_y = inner_rect.max.y - theme.spacing_md - padding;
            let div_btn_width = 28.0;
            let divisions = [BeatDivision::Eighth, BeatDivision::Quarter, BeatDivision::Half];
            let start_x = inner_rect.max.x - tap_width;

            for (i, div) in divisions.iter().enumerate() {
                let div_rect = Rect::from_min_size(
                    Pos2::new(start_x + i as f32 * (div_btn_width - 8.0) - 20.0, div_y),
                    Vec2::new(div_btn_width, theme.spacing_md),
                );
                let resp = ui.allocate_rect(div_rect, Sense::click());
                if resp.clicked() {
                    interactions.division_clicked = Some(*div);
                }
            }
        }

        // Drawing
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_md, theme.bg_secondary);

        // Tap button
        if self.show_tap_button {
            let tap_bg = if self.state.is_synced {
                theme.state_success.gamma_multiply(0.3)
            } else {
                theme.bg_tertiary
            };
            painter.rect_filled(tap_rect, theme.radius_sm, tap_bg);
            painter.text(
                tap_rect.center(),
                egui::Align2::CENTER_CENTER,
                "TAP",
                egui::FontId::proportional(theme.font_size_md),
                theme.text_primary,
            );

            // Tap count indicator
            let tap_count = self.state.tap_times.len();
            if tap_count > 0 {
                let dots_y = tap_rect.max.y - theme.spacing_xs;
                for i in 0..tap_count.min(8) {
                    let dot_x = tap_rect.min.x + theme.spacing_xs + i as f32 * 5.0;
                    painter.circle_filled(
                        Pos2::new(dot_x, dots_y),
                        2.0,
                        theme.primary,
                    );
                }
            }
        }

        // BPM display
        let bpm_text = format!("{:.1}", self.state.bpm);
        painter.text(
            bpm_rect.center(),
            egui::Align2::CENTER_CENTER,
            &bpm_text,
            egui::FontId::proportional(theme.font_size_2xl),
            theme.text_primary,
        );
        painter.text(
            Pos2::new(bpm_rect.center().x, bpm_rect.max.y - theme.spacing_xs),
            egui::Align2::CENTER_BOTTOM,
            "BPM",
            egui::FontId::proportional(theme.font_size_xs),
            theme.text_muted,
        );

        // +/- buttons
        if self.show_bpm_adjust {
            let btn_size = theme.spacing_lg;
            let minus_rect = Rect::from_min_size(
                Pos2::new(bpm_rect.min.x, bpm_rect.center().y - btn_size / 2.0),
                Vec2::splat(btn_size),
            );
            let plus_rect = Rect::from_min_size(
                Pos2::new(bpm_rect.max.x - btn_size, bpm_rect.center().y - btn_size / 2.0),
                Vec2::splat(btn_size),
            );

            painter.rect_filled(minus_rect, theme.radius_sm, theme.bg_tertiary);
            painter.text(minus_rect.center(), egui::Align2::CENTER_CENTER, "−", egui::FontId::proportional(theme.font_size_lg), theme.text_secondary);

            painter.rect_filled(plus_rect, theme.radius_sm, theme.bg_tertiary);
            painter.text(plus_rect.center(), egui::Align2::CENTER_CENTER, "+", egui::FontId::proportional(theme.font_size_lg), theme.text_secondary);
        }

        // Phase indicator
        if self.show_phase_indicator {
            let bar_rect = Rect::from_min_size(
                Pos2::new(phase_rect.min.x + theme.spacing_md + 2.0, phase_rect.center().y - 4.0),
                Vec2::new(phase_rect.width() - theme.spacing_md * 2.0 - 4.0, 8.0),
            );

            painter.rect_filled(bar_rect, 4.0, theme.bg_tertiary);

            // Beat markers (4 beats)
            for i in 0..4 {
                let t = i as f32 / 4.0;
                let x = bar_rect.min.x + t * bar_rect.width();
                painter.line_segment(
                    [Pos2::new(x, bar_rect.min.y), Pos2::new(x, bar_rect.max.y)],
                    Stroke::new(1.0, theme.border),
                );
            }

            // Phase position
            let phase_x = bar_rect.min.x + self.state.phase * bar_rect.width();
            let beat_in_bar = self.state.beat_count % 4;
            let phase_color = if beat_in_bar == 0 {
                theme.state_warning
            } else {
                theme.primary
            };
            painter.circle_filled(
                Pos2::new(phase_x, bar_rect.center().y),
                6.0,
                phase_color,
            );

            // Nudge arrows
            painter.text(
                Pos2::new(phase_rect.min.x + theme.spacing_xs, phase_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "◀",
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );
            painter.text(
                Pos2::new(phase_rect.max.x - theme.spacing_xs, phase_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                "▶",
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );
        }

        // Resync button
        let resync_bg = theme.bg_tertiary;
        painter.rect_filled(resync_rect, theme.radius_sm, resync_bg);
        painter.text(
            resync_rect.center(),
            egui::Align2::CENTER_CENTER,
            "SYNC",
            egui::FontId::proportional(theme.font_size_sm),
            theme.text_secondary,
        );

        // Division buttons
        if self.show_division {
            let div_y = inner_rect.max.y - theme.spacing_md - padding;
            let div_btn_width = 28.0;
            let divisions = [BeatDivision::Eighth, BeatDivision::Quarter, BeatDivision::Half];
            let start_x = inner_rect.max.x - tap_width;

            for (i, div) in divisions.iter().enumerate() {
                let div_rect = Rect::from_min_size(
                    Pos2::new(start_x + i as f32 * (div_btn_width - 8.0) - 20.0, div_y),
                    Vec2::new(div_btn_width, theme.spacing_md),
                );
                let is_active = self.state.division == *div;
                let bg = if is_active { theme.primary } else { theme.bg_tertiary };
                let text_color = if is_active { theme.primary_text } else { theme.text_muted };

                painter.rect_filled(div_rect, theme.radius_sm, bg);
                painter.text(
                    div_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    div.label(),
                    egui::FontId::proportional(theme.font_size_xs),
                    text_color,
                );
            }
        }

        // Border
        painter.rect_stroke(rect, theme.radius_md, Stroke::new(theme.border_width, theme.border), egui::StrokeKind::Inside);

        // Process events
        if interactions.tap_clicked {
            let time = ui.input(|i| i.time);
            event = Some(BeatSyncEvent::Tap(time));
        } else if interactions.resync_clicked {
            event = Some(BeatSyncEvent::Resync);
        } else if interactions.minus_clicked {
            event = Some(BeatSyncEvent::SetBpm((self.state.bpm - 1.0).max(20.0)));
        } else if interactions.plus_clicked {
            event = Some(BeatSyncEvent::SetBpm((self.state.bpm + 1.0).min(300.0)));
        } else if let Some(div) = interactions.division_clicked {
            event = Some(BeatSyncEvent::SetDivision(div));
        } else if interactions.nudge_left {
            event = Some(BeatSyncEvent::NudgePhase(-0.05));
        } else if interactions.nudge_right {
            event = Some(BeatSyncEvent::NudgePhase(0.05));
        }

        event
    }

    fn show_compact(self, ui: &mut Ui, rect: Rect, theme: &Theme) -> Option<BeatSyncEvent> {
        let mut event: Option<BeatSyncEvent> = None;

        // Tap area (left half)
        let tap_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width() * 0.4, rect.height()));
        let tap_resp = ui.allocate_rect(tap_rect, Sense::click());

        // BPM area (right)
        let bpm_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + rect.width() * 0.4, rect.min.y),
            Vec2::new(rect.width() * 0.6, rect.height()),
        );

        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_sm, theme.bg_secondary);

        // Tap section
        let tap_bg = if self.state.is_synced {
            theme.state_success.gamma_multiply(0.3)
        } else {
            theme.bg_tertiary
        };
        painter.rect_filled(tap_rect.shrink(2.0), theme.radius_sm, tap_bg);

        // Beat pulse indicator
        let pulse_size = 8.0 + (1.0 - self.state.phase) * 4.0;
        let pulse_alpha = (1.0 - self.state.phase).powf(2.0);
        painter.circle_filled(
            tap_rect.center(),
            pulse_size,
            theme.primary.gamma_multiply(pulse_alpha),
        );
        painter.text(
            tap_rect.center(),
            egui::Align2::CENTER_CENTER,
            "TAP",
            egui::FontId::proportional(theme.font_size_sm),
            theme.text_primary,
        );

        // BPM display
        let bpm_text = format!("{:.0}", self.state.bpm);
        painter.text(
            bpm_rect.center(),
            egui::Align2::CENTER_CENTER,
            &bpm_text,
            egui::FontId::proportional(theme.font_size_lg),
            theme.text_primary,
        );

        // Border
        painter.rect_stroke(rect, theme.radius_sm, Stroke::new(theme.border_width, theme.border), egui::StrokeKind::Inside);

        if tap_resp.clicked() {
            let time = ui.input(|i| i.time);
            event = Some(BeatSyncEvent::Tap(time));
        }

        event
    }
}
