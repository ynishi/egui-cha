//! ErrorConsole molecule - Collects and displays errors

use egui::{Color32, RichText, Ui};
use egui_cha::ViewCtx;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// An error entry with timestamp
#[derive(Clone, Debug)]
pub struct ErrorEntry {
    pub message: String,
    pub timestamp: Instant,
    pub level: ErrorLevel,
}

/// Error severity level
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ErrorLevel {
    #[default]
    Error,
    Warning,
    Info,
}

/// State for ErrorConsole (owned by parent)
#[derive(Default)]
pub struct ErrorConsoleState {
    errors: VecDeque<ErrorEntry>,
    max_entries: usize,
    auto_dismiss: Option<Duration>,
}

impl ErrorConsoleState {
    pub fn new() -> Self {
        Self {
            errors: VecDeque::new(),
            max_entries: 10,
            auto_dismiss: Some(Duration::from_secs(10)),
        }
    }

    /// Set maximum number of errors to keep
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set auto-dismiss duration (None to disable)
    pub fn with_auto_dismiss(mut self, duration: Option<Duration>) -> Self {
        self.auto_dismiss = duration;
        self
    }

    /// Push a new error
    pub fn push(&mut self, message: impl Into<String>) {
        self.push_with_level(message, ErrorLevel::Error);
    }

    /// Push a warning
    pub fn push_warning(&mut self, message: impl Into<String>) {
        self.push_with_level(message, ErrorLevel::Warning);
    }

    /// Push an info message
    pub fn push_info(&mut self, message: impl Into<String>) {
        self.push_with_level(message, ErrorLevel::Info);
    }

    /// Push with specific level
    pub fn push_with_level(&mut self, message: impl Into<String>, level: ErrorLevel) {
        self.errors.push_back(ErrorEntry {
            message: message.into(),
            timestamp: Instant::now(),
            level,
        });

        // Trim to max entries
        while self.errors.len() > self.max_entries {
            self.errors.pop_front();
        }
    }

    /// Remove expired errors (call this in update)
    pub fn cleanup(&mut self) {
        if let Some(duration) = self.auto_dismiss {
            let now = Instant::now();
            self.errors.retain(|e| now.duration_since(e.timestamp) < duration);
        }
    }

    /// Clear all errors
    pub fn clear(&mut self) {
        self.errors.clear();
    }

    /// Dismiss a specific error by index
    pub fn dismiss(&mut self, index: usize) {
        if index < self.errors.len() {
            self.errors.remove(index);
        }
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get error count
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Iterate over errors
    pub fn iter(&self) -> impl Iterator<Item = &ErrorEntry> {
        self.errors.iter()
    }

    /// Drain all errors (useful for batch processing)
    pub fn drain(&mut self) -> impl Iterator<Item = ErrorEntry> + '_ {
        self.errors.drain(..)
    }
}

/// Messages for ErrorConsole
#[derive(Clone, Debug)]
pub enum ErrorConsoleMsg {
    Dismiss(usize),
    DismissAll,
}

/// ErrorConsole component
pub struct ErrorConsole;

impl ErrorConsole {
    /// Get colors for error level based on theme
    fn level_colors(level: ErrorLevel, is_dark: bool) -> (Color32, Color32, &'static str) {
        match level {
            ErrorLevel::Error => {
                if is_dark {
                    (Color32::from_rgb(153, 27, 27), Color32::from_rgb(254, 202, 202), "✕")
                } else {
                    (Color32::from_rgb(254, 226, 226), Color32::from_rgb(153, 27, 27), "✕")
                }
            }
            ErrorLevel::Warning => {
                if is_dark {
                    (Color32::from_rgb(133, 77, 14), Color32::from_rgb(254, 240, 138), "⚠")
                } else {
                    (Color32::from_rgb(254, 249, 195), Color32::from_rgb(133, 77, 14), "⚠")
                }
            }
            ErrorLevel::Info => {
                if is_dark {
                    (Color32::from_rgb(30, 64, 175), Color32::from_rgb(191, 219, 254), "ℹ")
                } else {
                    (Color32::from_rgb(219, 234, 254), Color32::from_rgb(30, 64, 175), "ℹ")
                }
            }
        }
    }

    /// Get header color based on theme
    fn header_color(is_dark: bool) -> Color32 {
        if is_dark {
            Color32::from_rgb(248, 113, 113) // lighter red for dark mode
        } else {
            Color32::from_rgb(239, 68, 68)
        }
    }

    /// Show the error console (ViewCtx version)
    pub fn show<Msg>(
        ctx: &mut ViewCtx<'_, Msg>,
        state: &ErrorConsoleState,
        map_msg: impl Fn(ErrorConsoleMsg) -> Msg + Clone,
    ) {
        if state.is_empty() {
            return;
        }

        let is_dark = ctx.ui.ctx().style().visuals.dark_mode;

        // Collect dismiss clicks first
        let mut dismiss_index: Option<usize> = None;
        let mut clear_all = false;

        ctx.ui.vertical(|ui| {
            // Header with clear all button
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Errors ({})", state.len()))
                        .strong()
                        .color(Self::header_color(is_dark)),
                );
                ui.add_space(8.0);
                if ui.small_button("Clear All").clicked() {
                    clear_all = true;
                }
            });

            ui.add_space(4.0);

            // Error list
            for (index, entry) in state.iter().enumerate() {
                let (bg_color, text_color, icon) = Self::level_colors(entry.level, is_dark);

                egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).color(text_color));
                            ui.label(RichText::new(&entry.message).color(text_color));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("×").clicked() {
                                    dismiss_index = Some(index);
                                }
                            });
                        });
                    });

                ui.add_space(2.0);
            }
        });

        // Emit messages after UI rendering
        if clear_all {
            ctx.emit(map_msg(ErrorConsoleMsg::DismissAll));
        } else if let Some(index) = dismiss_index {
            ctx.emit(map_msg(ErrorConsoleMsg::Dismiss(index)));
        }
    }

    /// Show without ViewCtx (basic Ui version)
    pub fn show_ui(ui: &mut Ui, state: &ErrorConsoleState) -> Option<ErrorConsoleMsg> {
        if state.is_empty() {
            return None;
        }

        let is_dark = ui.ctx().style().visuals.dark_mode;
        let mut result = None;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Errors ({})", state.len()))
                        .strong()
                        .color(Self::header_color(is_dark)),
                );
                ui.add_space(8.0);
                if ui.small_button("Clear All").clicked() {
                    result = Some(ErrorConsoleMsg::DismissAll);
                }
            });

            ui.add_space(4.0);

            for (index, entry) in state.iter().enumerate() {
                let (bg_color, text_color, icon) = Self::level_colors(entry.level, is_dark);

                egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).color(text_color));
                            ui.label(RichText::new(&entry.message).color(text_color));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("×").clicked() && result.is_none() {
                                    result = Some(ErrorConsoleMsg::Dismiss(index));
                                }
                            });
                        });
                    });

                ui.add_space(2.0);
            }
        });

        result
    }
}
