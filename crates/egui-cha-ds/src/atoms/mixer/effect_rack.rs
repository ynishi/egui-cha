//! EffectRack - Audio effect chain display and control
//!
//! A rack-style component for displaying and managing a chain of audio effects.
//!
//! # Example
//! ```ignore
//! EffectRack::new(&model.effects)
//!     .show_with(ctx, |event| match event {
//!         RackEvent::Toggle(idx) => Msg::ToggleEffect(idx),
//!         RackEvent::Reorder(from, to) => Msg::ReorderEffects(from, to),
//!         RackEvent::Remove(idx) => Msg::RemoveEffect(idx),
//!         RackEvent::Select(idx) => Msg::SelectEffect(idx),
//!         RackEvent::ParamChange(idx, param, val) => Msg::SetParam(idx, param, val),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Effect rack events
#[derive(Clone, Debug, PartialEq)]
pub enum RackEvent {
    /// Effect enabled/disabled toggle
    Toggle(usize),
    /// Effect reordered (from_index, to_index)
    Reorder(usize, usize),
    /// Effect removed
    Remove(usize),
    /// Effect selected for editing
    Select(usize),
    /// Effect parameter changed (effect_index, param_index, value)
    ParamChange(usize, usize, f32),
    /// Add effect button clicked
    AddEffect,
}

/// Effect type category
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EffectCategory {
    /// Dynamics (compressor, limiter, gate)
    Dynamics,
    /// EQ and filters
    #[default]
    EQ,
    /// Time-based (delay, reverb)
    Time,
    /// Modulation (chorus, flanger, phaser)
    Modulation,
    /// Distortion and saturation
    Distortion,
    /// Utility (gain, panner)
    Utility,
}

impl EffectCategory {
    /// Get category color
    pub fn color(&self, theme: &Theme) -> Color32 {
        match self {
            EffectCategory::Dynamics => theme.state_success,
            EffectCategory::EQ => theme.primary,
            EffectCategory::Time => theme.secondary,
            EffectCategory::Modulation => Color32::from_rgb(200, 100, 255),
            EffectCategory::Distortion => theme.state_danger,
            EffectCategory::Utility => theme.text_muted,
        }
    }

    /// Get short label
    pub fn label(&self) -> &'static str {
        match self {
            EffectCategory::Dynamics => "DYN",
            EffectCategory::EQ => "EQ",
            EffectCategory::Time => "TIME",
            EffectCategory::Modulation => "MOD",
            EffectCategory::Distortion => "DIST",
            EffectCategory::Utility => "UTIL",
        }
    }
}

/// An effect parameter
#[derive(Clone, Debug)]
pub struct EffectParam {
    /// Parameter name
    pub name: String,
    /// Current value (0.0-1.0 normalized)
    pub value: f32,
    /// Display format (e.g., "{:.0}%", "{:.1}dB")
    pub format: Option<String>,
    /// Min/max for display
    pub range: (f32, f32),
}

impl EffectParam {
    /// Create a new parameter
    pub fn new(name: impl Into<String>, value: f32) -> Self {
        Self {
            name: name.into(),
            value: value.clamp(0.0, 1.0),
            format: None,
            range: (0.0, 100.0),
        }
    }

    /// Set display format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set display range
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.range = (min, max);
        self
    }

    /// Get formatted display value
    pub fn display_value(&self) -> String {
        let actual_value = self.range.0 + self.value * (self.range.1 - self.range.0);
        if let Some(ref fmt) = self.format {
            // Simple format substitution
            fmt.replace("{}", &format!("{:.1}", actual_value))
        } else {
            format!("{:.1}", actual_value)
        }
    }
}

/// An effect in the rack
#[derive(Clone, Debug)]
pub struct Effect {
    /// Effect name
    pub name: String,
    /// Effect category
    pub category: EffectCategory,
    /// Is effect enabled
    pub enabled: bool,
    /// Is effect bypassed (signal passes through unchanged)
    pub bypassed: bool,
    /// Effect parameters
    pub params: Vec<EffectParam>,
    /// Optional preset name
    pub preset: Option<String>,
}

impl Effect {
    /// Create a new effect
    pub fn new(name: impl Into<String>, category: EffectCategory) -> Self {
        Self {
            name: name.into(),
            category,
            enabled: true,
            bypassed: false,
            params: Vec::new(),
            preset: None,
        }
    }

    /// Add a parameter
    pub fn with_param(mut self, param: EffectParam) -> Self {
        self.params.push(param);
        self
    }

    /// Set preset name
    pub fn with_preset(mut self, preset: impl Into<String>) -> Self {
        self.preset = Some(preset.into());
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set bypassed state
    pub fn bypassed(mut self, bypassed: bool) -> Self {
        self.bypassed = bypassed;
        self
    }
}

/// Effect rack orientation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RackOrientation {
    /// Vertical stack (typical DAW layout)
    #[default]
    Vertical,
    /// Horizontal chain (pedalboard style)
    Horizontal,
}

/// Effect rack component
pub struct EffectRack<'a> {
    effects: &'a [Effect],
    orientation: RackOrientation,
    selected: Option<usize>,
    compact: bool,
    show_params: bool,
    show_chain_line: bool,
    draggable: bool,
    effect_height: f32,
    effect_width: f32,
}

impl<'a> EffectRack<'a> {
    /// Create a new effect rack
    pub fn new(effects: &'a [Effect]) -> Self {
        Self {
            effects,
            orientation: RackOrientation::Vertical,
            selected: None,
            compact: false,
            show_params: true,
            show_chain_line: true,
            draggable: true,
            effect_height: 64.0,
            effect_width: 120.0,
        }
    }

    /// Set orientation
    pub fn orientation(mut self, orientation: RackOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set selected effect index
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Use compact mode
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self.effect_height = 40.0;
        self.effect_width = 80.0;
        self
    }

    /// Show/hide parameters
    pub fn show_params(mut self, show: bool) -> Self {
        self.show_params = show;
        self
    }

    /// Show/hide chain connection line
    pub fn show_chain_line(mut self, show: bool) -> Self {
        self.show_chain_line = show;
        self
    }

    /// Enable/disable drag reordering
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Set effect card size
    pub fn effect_size(mut self, width: f32, height: f32) -> Self {
        self.effect_width = width;
        self.effect_height = height;
        self
    }

    /// TEA-style: Show rack and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(RackEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show rack, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<RackEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<RackEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        let spacing = theme.spacing_sm;
        let add_button_size = 32.0;

        // Calculate total size
        let (total_width, total_height) = match self.orientation {
            RackOrientation::Vertical => {
                let width = self.effect_width + spacing * 2.0;
                let height = self.effects.len() as f32 * (self.effect_height + spacing)
                    + add_button_size
                    + spacing;
                (width, height)
            }
            RackOrientation::Horizontal => {
                let width = self.effects.len() as f32 * (self.effect_width + spacing)
                    + add_button_size
                    + spacing;
                let height = self.effect_height + spacing * 2.0;
                (width, height)
            }
        };

        let (rect, _) = ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // First pass: collect all interactions
        struct EffectInfo {
            effect_rect: Rect,
            is_selected: bool,
            is_hovered: bool,
            is_enabled: bool,
            toggle_rect: Rect,
            toggle_hovered: bool,
            content_rect: Rect,
            category_color: Color32,
            bg_color: Color32,
            toggle_color: Color32,
            name_color: Color32,
            effect_idx: usize,
        }

        let mut effect_infos = Vec::new();
        let toggle_size = if self.compact { 12.0 } else { 16.0 };

        for (idx, effect) in self.effects.iter().enumerate() {
            let effect_rect = match self.orientation {
                RackOrientation::Vertical => Rect::from_min_size(
                    egui::pos2(
                        rect.min.x + spacing,
                        rect.min.y + spacing + idx as f32 * (self.effect_height + spacing),
                    ),
                    Vec2::new(self.effect_width, self.effect_height),
                ),
                RackOrientation::Horizontal => Rect::from_min_size(
                    egui::pos2(
                        rect.min.x + spacing + idx as f32 * (self.effect_width + spacing),
                        rect.min.y + spacing,
                    ),
                    Vec2::new(self.effect_width, self.effect_height),
                ),
            };

            let response = ui.allocate_rect(effect_rect, Sense::click());

            let is_selected = self.selected == Some(idx);
            let is_hovered = response.hovered();
            let is_enabled = effect.enabled && !effect.bypassed;

            let bg_color = if is_selected {
                theme.bg_tertiary
            } else if is_hovered {
                Color32::from_rgba_unmultiplied(
                    theme.bg_secondary.r(),
                    theme.bg_secondary.g(),
                    theme.bg_secondary.b(),
                    200,
                )
            } else {
                theme.bg_secondary
            };

            let category_color = if is_enabled {
                effect.category.color(&theme)
            } else {
                theme.text_muted
            };

            let content_rect = Rect::from_min_max(
                egui::pos2(effect_rect.min.x + 8.0, effect_rect.min.y + 4.0),
                egui::pos2(effect_rect.max.x - 4.0, effect_rect.max.y - 4.0),
            );

            let toggle_rect = Rect::from_min_size(
                egui::pos2(content_rect.min.x, content_rect.min.y),
                Vec2::splat(toggle_size),
            );

            let toggle_response = ui.allocate_rect(toggle_rect, Sense::click());
            if toggle_response.clicked() {
                event = Some(RackEvent::Toggle(idx));
            }

            let toggle_color = if is_enabled {
                theme.state_success
            } else {
                theme.text_muted
            };

            let name_color = if is_enabled {
                theme.text_primary
            } else {
                theme.text_muted
            };

            // Handle selection click
            if response.clicked() {
                event = Some(RackEvent::Select(idx));
            }

            // Handle right-click to remove
            if response.secondary_clicked() {
                event = Some(RackEvent::Remove(idx));
            }

            effect_infos.push(EffectInfo {
                effect_rect,
                is_selected,
                is_hovered,
                is_enabled,
                toggle_rect,
                toggle_hovered: toggle_response.hovered(),
                content_rect,
                category_color,
                bg_color,
                toggle_color,
                name_color,
                effect_idx: idx,
            });
        }

        // Add button interaction
        let add_button_rect = match self.orientation {
            RackOrientation::Vertical => Rect::from_center_size(
                egui::pos2(
                    rect.center().x,
                    rect.min.y
                        + self.effects.len() as f32 * (self.effect_height + spacing)
                        + spacing
                        + add_button_size / 2.0,
                ),
                Vec2::splat(add_button_size),
            ),
            RackOrientation::Horizontal => Rect::from_center_size(
                egui::pos2(
                    rect.min.x
                        + self.effects.len() as f32 * (self.effect_width + spacing)
                        + spacing
                        + add_button_size / 2.0,
                    rect.center().y,
                ),
                Vec2::splat(add_button_size),
            ),
        };

        let add_response = ui.allocate_rect(add_button_rect, Sense::click());
        let add_hovered = add_response.hovered();

        if add_response.clicked() {
            event = Some(RackEvent::AddEffect);
        }

        // Second pass: all painting
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_md, theme.bg_primary);

        // Draw chain connection line
        if self.show_chain_line && !self.effects.is_empty() {
            let chain_color = Color32::from_rgba_unmultiplied(
                theme.primary.r(),
                theme.primary.g(),
                theme.primary.b(),
                80,
            );

            match self.orientation {
                RackOrientation::Vertical => {
                    let x = rect.center().x;
                    let y1 = rect.min.y + spacing;
                    let y2 = rect.min.y
                        + self.effects.len() as f32 * (self.effect_height + spacing)
                        + spacing / 2.0;
                    painter.line_segment(
                        [egui::pos2(x, y1), egui::pos2(x, y2)],
                        Stroke::new(3.0, chain_color),
                    );
                }
                RackOrientation::Horizontal => {
                    let y = rect.center().y;
                    let x1 = rect.min.x + spacing;
                    let x2 = rect.min.x
                        + self.effects.len() as f32 * (self.effect_width + spacing)
                        + spacing / 2.0;
                    painter.line_segment(
                        [egui::pos2(x1, y), egui::pos2(x2, y)],
                        Stroke::new(3.0, chain_color),
                    );
                }
            }
        }

        // Draw effects using collected info
        for (info, effect) in effect_infos.iter().zip(self.effects.iter()) {
            // Effect card background
            painter.rect_filled(info.effect_rect, theme.radius_sm, info.bg_color);

            // Category color bar
            let bar_rect = Rect::from_min_size(info.effect_rect.min, Vec2::new(4.0, info.effect_rect.height()));
            painter.rect_filled(bar_rect, theme.radius_sm, info.category_color);

            // Draw toggle
            painter.circle_filled(info.toggle_rect.center(), toggle_size / 2.0 - 2.0, info.toggle_color);
            painter.circle_stroke(
                info.toggle_rect.center(),
                toggle_size / 2.0 - 2.0,
                Stroke::new(1.0, theme.border),
            );

            // Effect name
            painter.text(
                egui::pos2(info.content_rect.min.x + toggle_size + 4.0, info.content_rect.min.y + toggle_size / 2.0),
                egui::Align2::LEFT_CENTER,
                &effect.name,
                egui::FontId::proportional(if self.compact {
                    theme.font_size_xs
                } else {
                    theme.font_size_sm
                }),
                info.name_color,
            );

            // Category label
            if !self.compact {
                painter.text(
                    egui::pos2(info.content_rect.max.x - 4.0, info.content_rect.min.y + toggle_size / 2.0),
                    egui::Align2::RIGHT_CENTER,
                    effect.category.label(),
                    egui::FontId::proportional(theme.font_size_xs * 0.8),
                    info.category_color,
                );
            }

            // Preset name
            if let Some(ref preset) = effect.preset {
                if !self.compact {
                    painter.text(
                        egui::pos2(
                            info.content_rect.min.x + toggle_size + 4.0,
                            info.content_rect.min.y + toggle_size + 4.0,
                        ),
                        egui::Align2::LEFT_TOP,
                        preset,
                        egui::FontId::proportional(theme.font_size_xs * 0.85),
                        theme.text_muted,
                    );
                }
            }

            // Parameters (if not compact and show_params)
            if self.show_params && !self.compact && !effect.params.is_empty() {
                let params_y = info.content_rect.min.y + toggle_size + 8.0;
                let param_width = (info.content_rect.width() - 4.0) / effect.params.len().min(3) as f32;

                for (param_idx, param) in effect.params.iter().take(3).enumerate() {
                    let param_x = info.content_rect.min.x + param_idx as f32 * param_width;

                    // Parameter bar background
                    let bar_bg_rect = Rect::from_min_size(
                        egui::pos2(param_x, params_y + 12.0),
                        Vec2::new(param_width - 4.0, 4.0),
                    );
                    painter.rect_filled(bar_bg_rect, 2.0, theme.bg_primary);

                    // Parameter bar fill
                    let bar_fill_rect = Rect::from_min_size(
                        bar_bg_rect.min,
                        Vec2::new(bar_bg_rect.width() * param.value, bar_bg_rect.height()),
                    );
                    painter.rect_filled(bar_fill_rect, 2.0, info.category_color);

                    // Parameter name
                    painter.text(
                        egui::pos2(param_x, params_y),
                        egui::Align2::LEFT_TOP,
                        &param.name,
                        egui::FontId::proportional(theme.font_size_xs * 0.7),
                        theme.text_muted,
                    );
                }
            }

            // Bypassed indicator
            if effect.bypassed {
                painter.text(
                    info.effect_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "BYPASS",
                    egui::FontId::proportional(theme.font_size_xs),
                    Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                );
            }

            // Selection border
            if info.is_selected {
                painter.rect_stroke(
                    info.effect_rect,
                    theme.radius_sm,
                    Stroke::new(2.0, theme.primary),
                    egui::StrokeKind::Inside,
                );
            } else {
                painter.rect_stroke(
                    info.effect_rect,
                    theme.radius_sm,
                    Stroke::new(theme.border_width, theme.border),
                    egui::StrokeKind::Inside,
                );
            }
        }

        // Draw add button
        let add_bg_color = if add_hovered {
            theme.bg_tertiary
        } else {
            theme.bg_secondary
        };

        painter.rect_filled(add_button_rect, theme.radius_md, add_bg_color);
        painter.rect_stroke(
            add_button_rect,
            theme.radius_md,
            Stroke::new(
                theme.border_width,
                if add_hovered {
                    theme.primary
                } else {
                    theme.border
                },
            ),
            egui::StrokeKind::Inside,
        );

        // Plus sign
        let plus_color = if add_hovered {
            theme.primary
        } else {
            theme.text_muted
        };
        let center = add_button_rect.center();
        let half_size = 8.0;

        painter.line_segment(
            [
                egui::pos2(center.x - half_size, center.y),
                egui::pos2(center.x + half_size, center.y),
            ],
            Stroke::new(2.0, plus_color),
        );
        painter.line_segment(
            [
                egui::pos2(center.x, center.y - half_size),
                egui::pos2(center.x, center.y + half_size),
            ],
            Stroke::new(2.0, plus_color),
        );

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_md,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        event
    }
}
