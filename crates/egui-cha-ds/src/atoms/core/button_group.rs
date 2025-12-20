//! ButtonGroup atom - Radio-style button group with normalized output
//!
//! A group of mutually exclusive buttons (like radio buttons) that returns
//! a normalized 0.0-1.0 value based on selection position.
//!
//! # Example
//! ```ignore
//! // Wave selector (returns 0.0, 0.33, 0.67, 1.0)
//! ButtonGroup::new(&["Sin", "Saw", "Sqr", "Tri"])
//!     .show_with(ctx, model.wave_type, Msg::SetWaveType);
//!
//! // With icons
//! ButtonGroup::new(&["◐", "●", "◑"])
//!     .show_with(ctx, model.pan_mode, Msg::SetPanMode);
//! ```

use crate::Theme;
use egui::{Response, Sense, Ui, Vec2};
use egui_cha::ViewCtx;

/// Button group orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GroupOrientation {
    /// Horizontal layout (default)
    #[default]
    Horizontal,
    /// Vertical layout
    Vertical,
}

/// Button group size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GroupSize {
    /// Compact size
    Compact,
    /// Medium size (default)
    #[default]
    Medium,
    /// Large size
    Large,
}

/// A radio-style button group that returns normalized 0.0-1.0 values
pub struct ButtonGroup<'a> {
    labels: &'a [&'a str],
    orientation: GroupOrientation,
    size: GroupSize,
    disabled: bool,
    /// Whether to stretch to fill available width
    expand: bool,
}

impl<'a> ButtonGroup<'a> {
    /// Create a new button group with the given labels
    pub fn new(labels: &'a [&'a str]) -> Self {
        Self {
            labels,
            orientation: GroupOrientation::default(),
            size: GroupSize::default(),
            disabled: false,
            expand: false,
        }
    }

    /// Set the orientation
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Use vertical orientation
    pub fn vertical(mut self) -> Self {
        self.orientation = GroupOrientation::Vertical;
        self
    }

    /// Set the size variant
    pub fn size(mut self, size: GroupSize) -> Self {
        self.size = size;
        self
    }

    /// Use compact size
    pub fn compact(mut self) -> Self {
        self.size = GroupSize::Compact;
        self
    }

    /// Use large size
    pub fn large(mut self) -> Self {
        self.size = GroupSize::Large;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Expand to fill available width
    pub fn expand(mut self) -> Self {
        self.expand = true;
        self
    }

    /// Convert normalized value (0.0-1.0) to index
    fn value_to_index(&self, value: f64) -> usize {
        if self.labels.len() <= 1 {
            return 0;
        }
        let max_idx = self.labels.len() - 1;
        (value * max_idx as f64).round() as usize
    }

    /// Convert index to normalized value (0.0-1.0)
    fn index_to_value(&self, index: usize) -> f64 {
        if self.labels.len() <= 1 {
            return 0.0;
        }
        let max_idx = self.labels.len() - 1;
        index as f64 / max_idx as f64
    }

    /// TEA-style: Show button group with normalized value, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: f64,
        on_change: impl FnOnce(f64) -> Msg,
    ) {
        let mut current = value;
        let response = self.show_internal(ctx.ui, &mut current);

        if response.changed() {
            ctx.emit(on_change(current));
        }
    }

    /// Show button group (modifies normalized value in place)
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> Response {
        self.show_internal(ui, value)
    }

    /// Show and return selected index instead of normalized value
    pub fn show_index(self, ui: &mut Ui, index: &mut usize) -> Response {
        let labels_len = self.labels.len();
        let mut value = self.index_to_value(*index);
        let response = self.show_internal(ui, &mut value);
        if response.changed() {
            // Recalculate index from value
            if labels_len <= 1 {
                *index = 0;
            } else {
                let max_idx = labels_len - 1;
                *index = (value * max_idx as f64).round() as usize;
            }
        }
        response
    }

    fn show_internal(self, ui: &mut Ui, value: &mut f64) -> Response {
        let theme = Theme::current(ui.ctx());
        let selected_idx = self.value_to_index(*value);

        // Calculate button dimensions
        let (button_height, font_size, padding_h) = match self.size {
            GroupSize::Compact => (
                theme.spacing_md + theme.spacing_sm,
                theme.font_size_xs,
                theme.spacing_sm,
            ),
            GroupSize::Medium => (
                theme.spacing_lg + theme.spacing_sm,
                theme.font_size_sm,
                theme.spacing_md,
            ),
            GroupSize::Large => (
                theme.spacing_xl,
                theme.font_size_md,
                theme.spacing_lg,
            ),
        };

        // Calculate total size
        let available_width = if self.expand { ui.available_width() } else { 0.0 };

        let mut total_response: Option<Response> = None;
        let mut changed = false;

        match self.orientation {
            GroupOrientation::Horizontal => {
                ui.horizontal(|ui| {
                    let button_width = if self.expand && !self.labels.is_empty() {
                        available_width / self.labels.len() as f32
                    } else {
                        0.0 // Will be calculated per button
                    };

                    for (idx, label) in self.labels.iter().enumerate() {
                        let is_selected = idx == selected_idx;
                        let is_first = idx == 0;
                        let is_last = idx == self.labels.len() - 1;

                        let response = self.draw_button(
                            ui,
                            label,
                            is_selected,
                            is_first,
                            is_last,
                            button_width,
                            button_height,
                            font_size,
                            padding_h,
                            &theme,
                        );

                        if response.clicked() && !self.disabled {
                            *value = self.index_to_value(idx);
                            changed = true;
                        }

                        if let Some(ref mut total) = total_response {
                            *total = total.union(response);
                        } else {
                            total_response = Some(response);
                        }
                    }
                });
            }
            GroupOrientation::Vertical => {
                ui.vertical(|ui| {
                    for (idx, label) in self.labels.iter().enumerate() {
                        let is_selected = idx == selected_idx;
                        let is_first = idx == 0;
                        let is_last = idx == self.labels.len() - 1;

                        let button_width = if self.expand { available_width } else { 0.0 };

                        let response = self.draw_button(
                            ui,
                            label,
                            is_selected,
                            is_first,
                            is_last,
                            button_width,
                            button_height,
                            font_size,
                            padding_h,
                            &theme,
                        );

                        if response.clicked() && !self.disabled {
                            *value = self.index_to_value(idx);
                            changed = true;
                        }

                        if let Some(ref mut total) = total_response {
                            *total = total.union(response);
                        } else {
                            total_response = Some(response);
                        }
                    }
                });
            }
        }

        let mut response = total_response.unwrap_or_else(|| {
            ui.allocate_response(Vec2::ZERO, Sense::hover())
        });

        if changed {
            response.mark_changed();
        }

        response
    }

    fn draw_button(
        &self,
        ui: &mut Ui,
        label: &str,
        is_selected: bool,
        is_first: bool,
        is_last: bool,
        min_width: f32,
        height: f32,
        font_size: f32,
        padding_h: f32,
        theme: &Theme,
    ) -> Response {
        // Calculate text size for button width
        let text_width = ui.fonts_mut(|f| {
            f.glyph_width(&egui::FontId::proportional(font_size), 'M') * label.len() as f32
        });
        let button_width = if min_width > 0.0 {
            min_width
        } else {
            text_width + padding_h * 2.0
        };

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(button_width, height),
            if self.disabled { Sense::hover() } else { Sense::click() },
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Determine colors
            let (bg_color, text_color) = if self.disabled {
                (theme.bg_tertiary, theme.text_muted)
            } else if is_selected {
                (theme.primary, theme.primary_text)
            } else if response.hovered() {
                (theme.bg_tertiary, theme.text_primary)
            } else {
                (theme.bg_secondary, theme.text_secondary)
            };

            // Calculate corner radius (only round outer corners)
            let radius = theme.radius_sm;
            let r = radius as u8;
            let rounding = match (is_first, is_last, &self.orientation) {
                (true, true, _) => egui::CornerRadius::same(r),
                (true, false, GroupOrientation::Horizontal) => egui::CornerRadius {
                    nw: r,
                    sw: r,
                    ne: 0,
                    se: 0,
                },
                (false, true, GroupOrientation::Horizontal) => egui::CornerRadius {
                    nw: 0,
                    sw: 0,
                    ne: r,
                    se: r,
                },
                (true, false, GroupOrientation::Vertical) => egui::CornerRadius {
                    nw: r,
                    ne: r,
                    sw: 0,
                    se: 0,
                },
                (false, true, GroupOrientation::Vertical) => egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: r,
                    se: r,
                },
                _ => egui::CornerRadius::ZERO,
            };

            // Draw background
            painter.rect_filled(rect, rounding, bg_color);

            // Draw border
            if !is_selected {
                painter.rect_stroke(
                    rect,
                    rounding,
                    egui::Stroke::new(theme.border_width, theme.border),
                    egui::StrokeKind::Inside,
                );
            }

            // Draw text
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(font_size),
                text_color,
            );
        }

        response
    }
}
