//! Status indicator with animated states
//!
//! Visual indicator showing agent/system state with optional pulse/blink animations.
//!
//! # Example
//! ```ignore
//! // Factory methods (recommended)
//! StatusIndicator::active().show(ui);
//! StatusIndicator::error().show(ui);
//!
//! // With label (inline)
//! ui.horizontal(|ui| {
//!     StatusIndicator::active().size(8.0).show(ui);
//!     ui.label("Connected");
//! });
//!
//! // TEA style
//! StatusIndicator::active()
//!     .label("Online")
//!     .show_with(ctx, || Msg::StatusClicked);
//! ```

use crate::Theme;
use egui::{Color32, Response, Sense, Ui, Vec2};
use egui_cha::ViewCtx;

/// Status state with predefined colors and animations
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Status {
    /// Gray, no animation - system offline/disconnected
    #[default]
    Offline,
    /// Dim green, no animation - connected but idle
    Idle,
    /// Bright green, pulse animation - actively working
    Active,
    /// Blue, pulse animation - processing/busy
    Busy,
    /// Yellow, slow blink - needs attention
    Warning,
    /// Red, fast blink - error state
    Error,
}

/// Animation type for status indicator
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Animation {
    /// No animation (static)
    #[default]
    None,
    /// Smooth breathing effect (0.5 - 1.0 speed recommended)
    Pulse { speed: f32 },
    /// On/off toggle effect (2.0 - 4.0 speed recommended)
    Blink { speed: f32 },
}

/// Visual status indicator with optional animation
pub struct StatusIndicator {
    status: Status,
    size: Option<f32>,
    label: Option<String>,
    custom_color: Option<Color32>,
    custom_animation: Option<Animation>,
}

impl StatusIndicator {
    // =========================================================================
    // Factory methods (recommended)
    // =========================================================================

    /// Create an offline indicator (gray, static)
    pub fn offline() -> Self {
        Self::new(Status::Offline)
    }

    /// Create an idle indicator (dim green, static)
    pub fn idle() -> Self {
        Self::new(Status::Idle)
    }

    /// Create an active indicator (bright green, pulse)
    pub fn active() -> Self {
        Self::new(Status::Active)
    }

    /// Create a busy indicator (blue, pulse)
    pub fn busy() -> Self {
        Self::new(Status::Busy)
    }

    /// Create a warning indicator (yellow, slow blink)
    pub fn warning() -> Self {
        Self::new(Status::Warning)
    }

    /// Create an error indicator (red, fast blink)
    pub fn error() -> Self {
        Self::new(Status::Error)
    }

    // =========================================================================
    // Generic constructor
    // =========================================================================

    /// Create a status indicator with specified status
    pub fn new(status: Status) -> Self {
        Self {
            status,
            size: None,
            label: None,
            custom_color: None,
            custom_animation: None,
        }
    }

    /// Create a custom status indicator with specified color
    pub fn custom(color: Color32) -> Self {
        Self {
            status: Status::Idle,
            size: None,
            label: None,
            custom_color: Some(color),
            custom_animation: None,
        }
    }

    // =========================================================================
    // Builder methods
    // =========================================================================

    /// Set indicator size in pixels (default: theme.spacing_sm)
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// Set label text to display next to indicator
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Override the default color for this status
    pub fn color(mut self, color: Color32) -> Self {
        self.custom_color = Some(color);
        self
    }

    /// Override the default animation for this status
    pub fn animation(mut self, animation: Animation) -> Self {
        self.custom_animation = Some(animation);
        self
    }

    // =========================================================================
    // Display methods
    // =========================================================================

    /// Show the status indicator (egui style)
    pub fn show(self, ui: &mut Ui) -> Response {
        self.show_internal(ui)
    }

    /// Show the status indicator with click handler (TEA style)
    pub fn show_with<Msg>(self, ctx: &mut ViewCtx<'_, Msg>, on_click: impl FnOnce() -> Msg) {
        let response = self.show_internal(ctx.ui);
        if response.clicked() {
            ctx.emit(on_click());
        }
    }

    /// Show the status indicator and emit message on click (TEA style)
    pub fn on_click<Msg: Clone>(self, ctx: &mut ViewCtx<'_, Msg>, msg: Msg) -> bool {
        let response = self.show_internal(ctx.ui);
        if response.clicked() {
            ctx.emit(msg);
            true
        } else {
            false
        }
    }

    fn show_internal(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let time = ui.input(|i| i.time) as f32;

        // Get size from builder or use theme default
        let size = self.size.unwrap_or(theme.spacing_sm);

        // Determine color and animation based on status
        let (base_color, default_animation) = match self.status {
            Status::Offline => (theme.text_muted, Animation::None),
            Status::Idle => (theme.state_success.gamma_multiply(0.6), Animation::None),
            Status::Active => (theme.state_success, Animation::Pulse { speed: 1.0 }),
            Status::Busy => (theme.state_info, Animation::Pulse { speed: 1.5 }),
            Status::Warning => (theme.state_warning, Animation::Blink { speed: 2.0 }),
            Status::Error => (theme.state_danger, Animation::Blink { speed: 4.0 }),
        };

        let color = self.custom_color.unwrap_or(base_color);
        let animation = self.custom_animation.unwrap_or(default_animation);

        // Calculate animation multiplier
        let alpha_multiplier = match animation {
            Animation::None => 1.0,
            Animation::Pulse { speed } => {
                // Smooth sine wave: 0.5 to 1.0
                0.5 + 0.5 * (time * speed * std::f32::consts::TAU).sin()
            }
            Animation::Blink { speed } => {
                // Sharp on/off: 1.0 or 0.3
                if (time * speed).fract() < 0.5 {
                    1.0
                } else {
                    0.3
                }
            }
        };

        let animated_color = color.gamma_multiply(alpha_multiplier);

        // Allocate space
        let desired_size = if self.label.is_some() {
            // Will be handled by horizontal layout
            Vec2::new(size, size)
        } else {
            Vec2::new(size, size)
        };

        // Draw with or without label
        let response = if let Some(ref label_text) = self.label {
            ui.horizontal(|ui| {
                let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

                if ui.is_rect_visible(rect) {
                    let center = rect.center();
                    let radius = size / 2.0;

                    // Draw glow effect for animated states
                    if !matches!(animation, Animation::None) {
                        let glow_radius = radius * (1.0 + 0.3 * alpha_multiplier);
                        let glow_color = animated_color.gamma_multiply(0.3);
                        ui.painter().circle_filled(center, glow_radius, glow_color);
                    }

                    // Draw main circle
                    ui.painter()
                        .circle_filled(center, radius * 0.8, animated_color);

                    // Draw outline for offline state
                    if matches!(self.status, Status::Offline) {
                        ui.painter().circle_stroke(
                            center,
                            radius * 0.8,
                            egui::Stroke::new(1.0, theme.border),
                        );
                    }
                }

                ui.add_space(theme.spacing_xs / 2.0);
                ui.label(label_text);

                response
            })
            .inner
        } else {
            let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

            if ui.is_rect_visible(rect) {
                let center = rect.center();
                let radius = size / 2.0;

                // Draw glow effect for animated states
                if !matches!(animation, Animation::None) {
                    let glow_radius = radius * (1.0 + 0.3 * alpha_multiplier);
                    let glow_color = animated_color.gamma_multiply(0.3);
                    ui.painter().circle_filled(center, glow_radius, glow_color);
                }

                // Draw main circle
                ui.painter()
                    .circle_filled(center, radius * 0.8, animated_color);

                // Draw outline for offline state
                if matches!(self.status, Status::Offline) {
                    ui.painter().circle_stroke(
                        center,
                        radius * 0.8,
                        egui::Stroke::new(1.0, theme.border),
                    );
                }
            }

            response
        };

        // Request repaint if animating
        if !matches!(animation, Animation::None) {
            ui.ctx().request_repaint();
        }

        response
    }
}
