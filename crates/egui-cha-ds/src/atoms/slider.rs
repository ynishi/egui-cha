//! Slider atom

use egui::Ui;
use egui_cha::ViewCtx;
use std::ops::RangeInclusive;

/// A slider component
pub struct Slider<'a> {
    label: Option<&'a str>,
    range: RangeInclusive<f64>,
    step: Option<f64>,
    show_value: bool,
    disabled: bool,
}

impl<'a> Slider<'a> {
    pub fn new(range: RangeInclusive<f64>) -> Self {
        Self {
            label: None,
            range,
            step: None,
            show_value: true,
            disabled: false,
        }
    }

    /// Create slider for integer range
    pub fn int(range: RangeInclusive<i32>) -> Self {
        Self::new((*range.start() as f64)..=(*range.end() as f64)).step(1.0)
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// TEA-style: Show slider with immutable value, emit Msg on change
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: f64,
        on_change: impl FnOnce(f64) -> Msg,
    ) {
        let mut current = value;

        let mut slider = egui::Slider::new(&mut current, self.range.clone());

        if let Some(label) = self.label {
            slider = slider.text(label);
        }

        if let Some(step) = self.step {
            slider = slider.step_by(step);
        }

        if !self.show_value {
            slider = slider.show_value(false);
        }

        let response = ctx.ui.add_enabled(!self.disabled, slider);

        if response.changed() {
            ctx.emit(on_change(current));
        }
    }

    /// TEA-style for integers
    pub fn show_with_int<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        value: i32,
        on_change: impl FnOnce(i32) -> Msg,
    ) {
        self.show_with(ctx, value as f64, |v| on_change(v as i32));
    }

    /// Show slider (modifies value in place)
    pub fn show(self, ui: &mut Ui, value: &mut f64) -> bool {
        let mut slider = egui::Slider::new(value, self.range.clone());

        if let Some(label) = self.label {
            slider = slider.text(label);
        }

        if let Some(step) = self.step {
            slider = slider.step_by(step);
        }

        if !self.show_value {
            slider = slider.show_value(false);
        }

        let response = ui.add_enabled(!self.disabled, slider);
        response.changed()
    }
}
