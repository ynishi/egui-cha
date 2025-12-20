//! Preview - Video/Image preview display
//!
//! A preview component for displaying video frames, images, or thumbnails
//! with optional overlays for timecode, playback state, and labels.
//!
//! # Example
//! ```ignore
//! Preview::new(texture_id)
//!     .size(320.0, 180.0)
//!     .timecode("00:01:23:15")
//!     .playing(model.is_playing)
//!     .label("Clip A")
//!     .show_with(ctx, |event| match event {
//!         PreviewEvent::Click => Msg::TogglePlay,
//!         PreviewEvent::DoubleClick => Msg::ToggleFullscreen,
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, TextureId, Ui, Vec2};
use egui_cha::ViewCtx;

/// Preview events
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewEvent {
    /// Single click
    Click,
    /// Double click
    DoubleClick,
    /// Right click
    RightClick,
}

/// Aspect ratio presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectRatio {
    /// 16:9 widescreen
    Widescreen,
    /// 4:3 standard
    Standard,
    /// 1:1 square
    Square,
    /// 9:16 vertical/portrait
    Portrait,
    /// 21:9 ultrawide
    Ultrawide,
    /// Custom ratio (width / height)
    Custom(f32),
    /// Free (no constraint)
    Free,
}

impl AspectRatio {
    /// Get the ratio as width / height
    pub fn ratio(&self) -> Option<f32> {
        match self {
            AspectRatio::Widescreen => Some(16.0 / 9.0),
            AspectRatio::Standard => Some(4.0 / 3.0),
            AspectRatio::Square => Some(1.0),
            AspectRatio::Portrait => Some(9.0 / 16.0),
            AspectRatio::Ultrawide => Some(21.0 / 9.0),
            AspectRatio::Custom(r) => Some(*r),
            AspectRatio::Free => None,
        }
    }
}

/// Preview state indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewState {
    /// Idle/stopped
    #[default]
    Idle,
    /// Playing
    Playing,
    /// Paused
    Paused,
    /// Loading/buffering
    Loading,
    /// Error state
    Error,
    /// Live input
    Live,
}

/// Preview component
pub struct Preview<'a> {
    texture: Option<TextureId>,
    size: Vec2,
    aspect_ratio: AspectRatio,
    timecode: Option<&'a str>,
    label: Option<&'a str>,
    state: PreviewState,
    show_border: bool,
    show_state_icon: bool,
    selected: bool,
    placeholder_color: Color32,
}

impl<'a> Preview<'a> {
    /// Create a new preview with texture
    pub fn new(texture: TextureId) -> Self {
        Self {
            texture: Some(texture),
            size: Vec2::new(320.0, 180.0),
            aspect_ratio: AspectRatio::Widescreen,
            timecode: None,
            label: None,
            state: PreviewState::Idle,
            show_border: true,
            show_state_icon: true,
            selected: false,
            placeholder_color: Color32::from_rgb(30, 30, 35),
        }
    }

    /// Create an empty preview (placeholder)
    pub fn empty() -> Self {
        Self {
            texture: None,
            size: Vec2::new(320.0, 180.0),
            aspect_ratio: AspectRatio::Widescreen,
            timecode: None,
            label: None,
            state: PreviewState::Idle,
            show_border: true,
            show_state_icon: false,
            selected: false,
            placeholder_color: Color32::from_rgb(30, 30, 35),
        }
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set aspect ratio
    pub fn aspect_ratio(mut self, ratio: AspectRatio) -> Self {
        self.aspect_ratio = ratio;
        self
    }

    /// Set timecode display
    pub fn timecode(mut self, tc: &'a str) -> Self {
        self.timecode = Some(tc);
        self
    }

    /// Set label
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set playback state
    pub fn state(mut self, state: PreviewState) -> Self {
        self.state = state;
        self
    }

    /// Convenience: set playing state
    pub fn playing(mut self, playing: bool) -> Self {
        self.state = if playing {
            PreviewState::Playing
        } else {
            PreviewState::Paused
        };
        self
    }

    /// Show/hide border
    pub fn show_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    /// Show/hide state icon
    pub fn show_state_icon(mut self, show: bool) -> Self {
        self.show_state_icon = show;
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set placeholder color
    pub fn placeholder_color(mut self, color: Color32) -> Self {
        self.placeholder_color = color;
        self
    }

    /// TEA-style: Show preview and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(PreviewEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show preview, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<PreviewEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<PreviewEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        // Calculate actual size based on aspect ratio
        let actual_size = if let Some(ratio) = self.aspect_ratio.ratio() {
            let height_from_width = self.size.x / ratio;
            let width_from_height = self.size.y * ratio;

            if height_from_width <= self.size.y {
                Vec2::new(self.size.x, height_from_width)
            } else {
                Vec2::new(width_from_height, self.size.y)
            }
        } else {
            self.size
        };

        let (rect, response) = ui.allocate_exact_size(actual_size, Sense::click());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Handle events
        if response.double_clicked() {
            event = Some(PreviewEvent::DoubleClick);
        } else if response.clicked() {
            event = Some(PreviewEvent::Click);
        } else if response.secondary_clicked() {
            event = Some(PreviewEvent::RightClick);
        }

        let painter = ui.painter();

        // Draw background/placeholder
        painter.rect_filled(rect, theme.radius_sm, self.placeholder_color);

        // Draw texture if available
        if let Some(texture) = self.texture {
            painter.image(
                texture,
                rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Draw placeholder icon (film frame)
            let icon_size = actual_size.min_elem() * 0.3;
            let center = rect.center();

            // Simple film icon
            let icon_rect = Rect::from_center_size(center, Vec2::splat(icon_size));
            painter.rect_stroke(
                icon_rect,
                theme.radius_sm * 0.5,
                Stroke::new(2.0, theme.text_muted),
                egui::StrokeKind::Inside,
            );

            // Film perforations
            let perf_count = 3;
            let perf_size = icon_size * 0.15;
            for i in 0..perf_count {
                let y = icon_rect.min.y + icon_size * (0.2 + 0.3 * i as f32);
                // Left perforations
                painter.rect_filled(
                    Rect::from_center_size(
                        egui::pos2(icon_rect.min.x - perf_size * 0.6, y),
                        Vec2::splat(perf_size),
                    ),
                    perf_size * 0.2,
                    theme.text_muted,
                );
                // Right perforations
                painter.rect_filled(
                    Rect::from_center_size(
                        egui::pos2(icon_rect.max.x + perf_size * 0.6, y),
                        Vec2::splat(perf_size),
                    ),
                    perf_size * 0.2,
                    theme.text_muted,
                );
            }
        }

        // Draw state icon overlay
        if self.show_state_icon && self.state != PreviewState::Idle {
            let icon_size = 24.0;
            let icon_center = rect.center();

            // Semi-transparent background
            painter.circle_filled(
                icon_center,
                icon_size,
                Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            );

            match self.state {
                PreviewState::Playing => {
                    // Pause icon (two bars)
                    let bar_width = 4.0;
                    let bar_height = 14.0;
                    let gap = 4.0;

                    painter.rect_filled(
                        Rect::from_center_size(
                            egui::pos2(icon_center.x - gap, icon_center.y),
                            Vec2::new(bar_width, bar_height),
                        ),
                        1.0,
                        Color32::WHITE,
                    );
                    painter.rect_filled(
                        Rect::from_center_size(
                            egui::pos2(icon_center.x + gap, icon_center.y),
                            Vec2::new(bar_width, bar_height),
                        ),
                        1.0,
                        Color32::WHITE,
                    );
                }
                PreviewState::Paused => {
                    // Play icon (triangle)
                    let size = 10.0;
                    let points = vec![
                        egui::pos2(icon_center.x - size * 0.5, icon_center.y - size),
                        egui::pos2(icon_center.x - size * 0.5, icon_center.y + size),
                        egui::pos2(icon_center.x + size, icon_center.y),
                    ];
                    painter.add(egui::Shape::convex_polygon(
                        points,
                        Color32::WHITE,
                        Stroke::NONE,
                    ));
                }
                PreviewState::Loading => {
                    // Loading spinner (simplified as rotating arc)
                    let time = ui.input(|i| i.time) as f32;
                    let angle = time * 3.0;
                    let arc_radius = 8.0;

                    for i in 0..8 {
                        let a = angle + i as f32 * std::f32::consts::PI / 4.0;
                        let alpha = ((7 - i) as f32 / 7.0 * 200.0) as u8;
                        let dot_pos = egui::pos2(
                            icon_center.x + a.cos() * arc_radius,
                            icon_center.y + a.sin() * arc_radius,
                        );
                        painter.circle_filled(
                            dot_pos,
                            2.0,
                            Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                        );
                    }

                    ui.ctx().request_repaint();
                }
                PreviewState::Error => {
                    // X icon
                    let size = 8.0;
                    painter.line_segment(
                        [
                            egui::pos2(icon_center.x - size, icon_center.y - size),
                            egui::pos2(icon_center.x + size, icon_center.y + size),
                        ],
                        Stroke::new(3.0, theme.state_danger),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(icon_center.x + size, icon_center.y - size),
                            egui::pos2(icon_center.x - size, icon_center.y + size),
                        ],
                        Stroke::new(3.0, theme.state_danger),
                    );
                }
                PreviewState::Live => {
                    // LIVE indicator
                    painter.circle_filled(icon_center, 6.0, theme.state_danger);
                }
                PreviewState::Idle => {}
            }
        }

        // Draw label (bottom left)
        if let Some(label) = self.label {
            let label_bg = Rect::from_min_size(
                egui::pos2(rect.min.x, rect.max.y - 20.0),
                Vec2::new(rect.width(), 20.0),
            );
            painter.rect_filled(
                label_bg,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            );
            painter.text(
                egui::pos2(rect.min.x + 4.0, rect.max.y - 10.0),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(theme.font_size_xs),
                Color32::WHITE,
            );
        }

        // Draw timecode (top right)
        if let Some(tc) = self.timecode {
            let tc_bg = Rect::from_min_size(
                egui::pos2(rect.max.x - 70.0, rect.min.y),
                Vec2::new(70.0, 18.0),
            );
            painter.rect_filled(
                tc_bg,
                0.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            );
            painter.text(
                egui::pos2(rect.max.x - 4.0, rect.min.y + 9.0),
                egui::Align2::RIGHT_CENTER,
                tc,
                egui::FontId::monospace(theme.font_size_xs),
                Color32::WHITE,
            );
        }

        // Draw LIVE badge if live
        if self.state == PreviewState::Live {
            let badge_rect = Rect::from_min_size(
                egui::pos2(rect.min.x + 4.0, rect.min.y + 4.0),
                Vec2::new(36.0, 16.0),
            );
            painter.rect_filled(badge_rect, theme.radius_sm * 0.5, theme.state_danger);
            painter.text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                "LIVE",
                egui::FontId::proportional(theme.font_size_xs * 0.9),
                Color32::WHITE,
            );
        }

        // Draw border
        if self.show_border {
            let border_color = if self.selected {
                theme.primary
            } else if response.hovered() {
                theme.border_focus
            } else {
                theme.border
            };
            let border_width = if self.selected { 2.0 } else { theme.border_width };

            painter.rect_stroke(
                rect,
                theme.radius_sm,
                Stroke::new(border_width, border_color),
                egui::StrokeKind::Inside,
            );
        }

        event
    }
}
