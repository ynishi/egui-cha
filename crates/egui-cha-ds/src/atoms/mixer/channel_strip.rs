//! ChannelStrip - Mixer channel strip with fader, meter, and controls
//!
//! A complete mixer channel with volume fader, level meter, mute/solo buttons,
//! pan control, and label. Commonly seen in DAWs and audio mixers.
//!
//! # Example
//! ```ignore
//! ChannelStrip::new("Drums")
//!     .volume(model.volume)  // 0.0-1.0
//!     .pan(model.pan)        // -1.0 to 1.0
//!     .level(model.peak_level)
//!     .mute(model.muted)
//!     .solo(model.soloed)
//!     .show_with(ctx, |event| match event {
//!         ChannelEvent::VolumeChange(v) => Msg::SetVolume(v),
//!         ChannelEvent::PanChange(p) => Msg::SetPan(p),
//!         ChannelEvent::Mute => Msg::ToggleMute,
//!         ChannelEvent::Solo => Msg::ToggleSolo,
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Channel strip events
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChannelEvent {
    /// Volume changed (0.0 - 1.0)
    VolumeChange(f32),
    /// Pan changed (-1.0 to 1.0)
    PanChange(f32),
    /// Mute button clicked
    Mute,
    /// Solo button clicked
    Solo,
    /// Channel selected
    Select,
}

/// Channel strip component
pub struct ChannelStrip<'a> {
    label: &'a str,
    volume: f32,
    pan: f32,
    level: f32,
    level_right: Option<f32>,
    muted: bool,
    soloed: bool,
    selected: bool,
    color: Option<Color32>,
    width: f32,
    show_pan: bool,
    show_meter: bool,
    compact: bool,
}

impl<'a> ChannelStrip<'a> {
    /// Create a new channel strip
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            volume: 0.75,
            pan: 0.0,
            level: 0.0,
            level_right: None,
            muted: false,
            soloed: false,
            selected: false,
            color: None,
            width: 60.0,
            show_pan: true,
            show_meter: true,
            compact: false,
        }
    }

    /// Set volume (0.0 - 1.0)
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set pan (-1.0 left to 1.0 right)
    pub fn pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    /// Set level meter value (0.0 - 1.0)
    pub fn level(mut self, level: f32) -> Self {
        self.level = level.clamp(0.0, 1.0);
        self
    }

    /// Set stereo level (left, right)
    pub fn stereo_level(mut self, left: f32, right: f32) -> Self {
        self.level = left.clamp(0.0, 1.0);
        self.level_right = Some(right.clamp(0.0, 1.0));
        self
    }

    /// Set mute state
    pub fn mute(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    /// Set solo state
    pub fn solo(mut self, soloed: bool) -> Self {
        self.soloed = soloed;
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set channel color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Show/hide pan control
    pub fn show_pan(mut self, show: bool) -> Self {
        self.show_pan = show;
        self
    }

    /// Show/hide level meter
    pub fn show_meter(mut self, show: bool) -> Self {
        self.show_meter = show;
        self
    }

    /// Compact mode
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// TEA-style: Show channel strip and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(ChannelEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show channel strip, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<ChannelEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<ChannelEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        let channel_color = self.color.unwrap_or(theme.primary);

        // Calculate heights
        let label_height = 20.0;
        let button_height = if self.compact { 18.0 } else { 22.0 };
        let pan_height = if self.show_pan { 24.0 } else { 0.0 };
        let fader_height = if self.compact { 100.0 } else { 140.0 };
        let meter_width = if self.show_meter { 8.0 } else { 0.0 };
        let spacing = theme.spacing_xs;

        let total_height = label_height + spacing
            + button_height * 2.0 + spacing * 2.0
            + pan_height + spacing
            + fader_height + spacing
            + label_height; // bottom label

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(self.width, total_height),
            Sense::click(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Handle selection
        if response.clicked() {
            event = Some(ChannelEvent::Select);
        }

        let mut y = rect.min.y;

        // Calculate all rects first
        let label_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, y),
            Vec2::new(self.width, label_height),
        );
        y += label_height + spacing;

        let mute_rect = Rect::from_min_size(
            egui::pos2(rect.min.x + 4.0, y),
            Vec2::new(self.width - 8.0, button_height),
        );
        y += button_height + spacing;

        let solo_rect = Rect::from_min_size(
            egui::pos2(rect.min.x + 4.0, y),
            Vec2::new(self.width - 8.0, button_height),
        );
        y += button_height + spacing;

        let pan_rect = if self.show_pan {
            let r = Rect::from_min_size(
                egui::pos2(rect.min.x + 4.0, y),
                Vec2::new(self.width - 8.0, pan_height),
            );
            y += pan_height + spacing;
            Some(r)
        } else {
            None
        };

        let fader_area_rect = Rect::from_min_size(
            egui::pos2(rect.min.x + 4.0, y),
            Vec2::new(self.width - 8.0, fader_height),
        );

        let meter_total_width = if self.level_right.is_some() {
            meter_width * 2.0 + 2.0
        } else {
            meter_width
        };

        let fader_width = fader_area_rect.width() - meter_total_width - 4.0;
        let fader_rect = Rect::from_min_size(
            fader_area_rect.min,
            Vec2::new(fader_width, fader_height),
        );

        y += fader_height + spacing;
        let vol_label_y = y;

        // First pass: handle interactions
        let mute_response = ui.allocate_rect(mute_rect, Sense::click());
        if mute_response.clicked() {
            event = Some(ChannelEvent::Mute);
        }

        let solo_response = ui.allocate_rect(solo_rect, Sense::click());
        if solo_response.clicked() {
            event = Some(ChannelEvent::Solo);
        }

        let pan_response = pan_rect.map(|r| {
            let resp = ui.allocate_rect(r, Sense::click_and_drag());
            if resp.dragged() {
                let delta = resp.drag_delta().x / (r.width() * 0.5);
                let new_pan = (self.pan + delta).clamp(-1.0, 1.0);
                event = Some(ChannelEvent::PanChange(new_pan));
            }
            if resp.double_clicked() {
                event = Some(ChannelEvent::PanChange(0.0));
            }
            resp
        });

        let fader_response = ui.allocate_rect(fader_rect, Sense::click_and_drag());
        if fader_response.dragged() {
            let delta = -fader_response.drag_delta().y / fader_rect.height();
            let new_vol = (self.volume + delta).clamp(0.0, 1.0);
            event = Some(ChannelEvent::VolumeChange(new_vol));
        }

        // Second pass: draw everything
        let painter = ui.painter();

        // Background
        let bg_color = if self.selected {
            Color32::from_rgba_unmultiplied(
                channel_color.r(),
                channel_color.g(),
                channel_color.b(),
                30,
            )
        } else {
            theme.bg_secondary
        };
        painter.rect_filled(rect, theme.radius_sm, bg_color);

        // Top label with color indicator
        let color_bar = Rect::from_min_size(
            label_rect.min,
            Vec2::new(self.width, 3.0),
        );
        painter.rect_filled(color_bar, theme.radius_sm, channel_color);

        painter.text(
            egui::pos2(label_rect.center().x, label_rect.center().y + 2.0),
            egui::Align2::CENTER_CENTER,
            self.label,
            egui::FontId::proportional(theme.font_size_xs),
            theme.text_primary,
        );

        // Mute button
        let mute_bg = if self.muted {
            theme.state_danger
        } else if mute_response.hovered() {
            theme.bg_tertiary
        } else {
            theme.bg_primary
        };
        painter.rect_filled(mute_rect, theme.radius_sm * 0.5, mute_bg);
        painter.text(
            mute_rect.center(),
            egui::Align2::CENTER_CENTER,
            "M",
            egui::FontId::proportional(theme.font_size_xs),
            if self.muted { Color32::WHITE } else { theme.text_secondary },
        );

        // Solo button
        let solo_bg = if self.soloed {
            theme.state_warning
        } else if solo_response.hovered() {
            theme.bg_tertiary
        } else {
            theme.bg_primary
        };
        painter.rect_filled(solo_rect, theme.radius_sm * 0.5, solo_bg);
        painter.text(
            solo_rect.center(),
            egui::Align2::CENTER_CENTER,
            "S",
            egui::FontId::proportional(theme.font_size_xs),
            if self.soloed { Color32::BLACK } else { theme.text_secondary },
        );

        // Pan control
        if let Some(pan_rect) = pan_rect {
            let pan_hovered = pan_response.as_ref().map(|r| r.hovered()).unwrap_or(false);
            let _ = pan_hovered; // Could use for highlighting

            painter.rect_filled(pan_rect, theme.radius_sm * 0.5, theme.bg_primary);

            let pan_center = pan_rect.center().x;
            let pan_x = pan_center + self.pan * (pan_rect.width() * 0.4);

            // Center line
            painter.line_segment(
                [
                    egui::pos2(pan_center, pan_rect.min.y + 4.0),
                    egui::pos2(pan_center, pan_rect.max.y - 4.0),
                ],
                Stroke::new(1.0, theme.text_muted),
            );

            // Pan position indicator
            painter.circle_filled(
                egui::pos2(pan_x, pan_rect.center().y),
                6.0,
                channel_color,
            );

            // L/R labels
            painter.text(
                egui::pos2(pan_rect.min.x + 4.0, pan_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "L",
                egui::FontId::proportional(theme.font_size_xs * 0.8),
                theme.text_muted,
            );
            painter.text(
                egui::pos2(pan_rect.max.x - 4.0, pan_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                "R",
                egui::FontId::proportional(theme.font_size_xs * 0.8),
                theme.text_muted,
            );
        }

        // Draw meter(s)
        if self.show_meter {
            let meter_x = fader_area_rect.max.x - meter_total_width;

            let meter_rect = Rect::from_min_size(
                egui::pos2(meter_x, fader_area_rect.min.y),
                Vec2::new(meter_width, fader_height),
            );
            self.draw_meter(painter, meter_rect, self.level, &theme);

            if let Some(right) = self.level_right {
                let right_rect = Rect::from_min_size(
                    egui::pos2(meter_x + meter_width + 2.0, fader_area_rect.min.y),
                    Vec2::new(meter_width, fader_height),
                );
                self.draw_meter(painter, right_rect, right, &theme);
            }
        }

        // Fader track
        let track_width = 6.0;
        let track_rect = Rect::from_center_size(
            fader_rect.center(),
            Vec2::new(track_width, fader_rect.height() - 20.0),
        );
        painter.rect_filled(track_rect, 2.0, theme.bg_primary);

        // Fader fill
        let fill_height = track_rect.height() * self.volume;
        let fill_rect = Rect::from_min_max(
            egui::pos2(track_rect.min.x, track_rect.max.y - fill_height),
            track_rect.max,
        );
        let fill_color = if self.muted {
            theme.text_muted
        } else {
            channel_color
        };
        painter.rect_filled(fill_rect, 2.0, fill_color);

        // Fader thumb
        let thumb_y = track_rect.max.y - self.volume * track_rect.height();
        let thumb_rect = Rect::from_center_size(
            egui::pos2(fader_rect.center().x, thumb_y),
            Vec2::new(fader_width - 4.0, 16.0),
        );

        let thumb_color = if fader_response.hovered() || fader_response.dragged() {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        painter.rect_filled(thumb_rect, theme.radius_sm * 0.5, thumb_color);

        // Thumb grip line
        painter.line_segment(
            [
                egui::pos2(thumb_rect.min.x + 4.0, thumb_rect.center().y),
                egui::pos2(thumb_rect.max.x - 4.0, thumb_rect.center().y),
            ],
            Stroke::new(1.0, theme.bg_primary),
        );

        // Volume display
        let vol_db = if self.volume > 0.0 {
            20.0 * self.volume.log10()
        } else {
            -60.0
        };
        let vol_text = if vol_db <= -60.0 {
            "-inf".to_string()
        } else {
            format!("{:.1}", vol_db)
        };

        painter.text(
            egui::pos2(rect.center().x, vol_label_y + label_height / 2.0),
            egui::Align2::CENTER_CENTER,
            vol_text,
            egui::FontId::monospace(theme.font_size_xs),
            theme.text_secondary,
        );

        // Border
        let border_color = if self.selected {
            channel_color
        } else {
            theme.border
        };
        painter.rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(if self.selected { 2.0 } else { theme.border_width }, border_color),
            egui::StrokeKind::Inside,
        );

        event
    }

    fn draw_meter(&self, painter: &egui::Painter, rect: Rect, level: f32, theme: &Theme) {
        // Background
        painter.rect_filled(rect, 2.0, theme.bg_primary);

        // Level fill with gradient colors
        let fill_height = rect.height() * level;
        let fill_rect = Rect::from_min_max(
            egui::pos2(rect.min.x, rect.max.y - fill_height),
            rect.max,
        );

        // Color based on level
        let color = if level > 0.95 {
            theme.state_danger // Clipping
        } else if level > 0.8 {
            theme.state_warning // Hot
        } else {
            theme.state_success // Normal
        };

        painter.rect_filled(fill_rect, 2.0, color);

        // Peak indicator line
        if level > 0.0 {
            let peak_y = rect.max.y - level * rect.height();
            painter.line_segment(
                [
                    egui::pos2(rect.min.x, peak_y),
                    egui::pos2(rect.max.x, peak_y),
                ],
                Stroke::new(1.0, Color32::WHITE),
            );
        }
    }
}
