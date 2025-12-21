//! Extras wrappers for egui_extras with theme integration
//!
//! Provides Strip layout component.
//!
//! # Example
//! ```ignore
//! // Strip layout
//! Strip::horizontal()
//!     .exact(100.0)
//!     .remainder()
//!     .show(ui, |i, ui| { ... });
//! ```

use egui::Ui;

/// Column width specification (re-exported for convenience)
#[derive(Clone, Copy, Default)]
pub enum ColumnWidth {
    #[default]
    Auto,
    Fixed(f32),
    Initial(f32),
    Remainder,
}

/// Strip layout helper with theme integration
pub struct Strip {
    sizes: Vec<StripSize>,
    direction: StripDirection,
}

/// Size specification for strip cells
#[derive(Clone, Copy)]
pub enum StripSize {
    /// Exact size in pixels
    Exact(f32),
    /// Initial size, resizable
    Initial(f32),
    /// Relative size (ratio)
    Relative(f32),
    /// Take remaining space
    Remainder,
}

/// Strip direction
#[derive(Clone, Copy, Default)]
pub enum StripDirection {
    #[default]
    Horizontal,
    Vertical,
}

impl Strip {
    /// Create a new horizontal strip
    pub fn horizontal() -> Self {
        Self {
            sizes: Vec::new(),
            direction: StripDirection::Horizontal,
        }
    }

    /// Create a new vertical strip
    pub fn vertical() -> Self {
        Self {
            sizes: Vec::new(),
            direction: StripDirection::Vertical,
        }
    }

    /// Add a cell with exact size
    pub fn exact(mut self, size: f32) -> Self {
        self.sizes.push(StripSize::Exact(size));
        self
    }

    /// Add a cell with initial size (resizable)
    pub fn initial(mut self, size: f32) -> Self {
        self.sizes.push(StripSize::Initial(size));
        self
    }

    /// Add a cell with relative size
    pub fn relative(mut self, ratio: f32) -> Self {
        self.sizes.push(StripSize::Relative(ratio));
        self
    }

    /// Add a cell that takes remaining space
    pub fn remainder(mut self) -> Self {
        self.sizes.push(StripSize::Remainder);
        self
    }

    /// Show the strip with content functions
    pub fn show<R>(self, ui: &mut Ui, mut add_contents: impl FnMut(usize, &mut Ui) -> R) -> Vec<R> {
        let mut results = Vec::new();

        match self.direction {
            StripDirection::Horizontal => {
                let mut builder = egui_extras::StripBuilder::new(ui);
                for size in &self.sizes {
                    builder = match size {
                        StripSize::Exact(s) => builder.size(egui_extras::Size::exact(*s)),
                        StripSize::Initial(s) => builder.size(egui_extras::Size::initial(*s)),
                        StripSize::Relative(r) => builder.size(egui_extras::Size::relative(*r)),
                        StripSize::Remainder => builder.size(egui_extras::Size::remainder()),
                    };
                }
                builder.horizontal(|mut strip| {
                    for i in 0..self.sizes.len() {
                        strip.cell(|ui| {
                            results.push(add_contents(i, ui));
                        });
                    }
                });
            }
            StripDirection::Vertical => {
                let mut builder = egui_extras::StripBuilder::new(ui);
                for size in &self.sizes {
                    builder = match size {
                        StripSize::Exact(s) => builder.size(egui_extras::Size::exact(*s)),
                        StripSize::Initial(s) => builder.size(egui_extras::Size::initial(*s)),
                        StripSize::Relative(r) => builder.size(egui_extras::Size::relative(*r)),
                        StripSize::Remainder => builder.size(egui_extras::Size::remainder()),
                    };
                }
                builder.vertical(|mut strip| {
                    for i in 0..self.sizes.len() {
                        strip.cell(|ui| {
                            results.push(add_contents(i, ui));
                        });
                    }
                });
            }
        }

        results
    }
}

/// Re-export egui_extras for advanced usage
pub mod raw {
    pub use egui_extras::*;
}
