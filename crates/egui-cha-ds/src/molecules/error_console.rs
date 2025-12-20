//! ErrorConsole molecule - Collects and displays errors

use crate::atoms::icons;
use crate::Theme;
use egui::{Color32, FontFamily, RichText, Ui};
use egui_cha::{Severity, ViewCtx};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// An error entry with timestamp
#[derive(Clone, Debug)]
pub struct ErrorEntry {
    pub message: String,
    pub timestamp: Instant,
    pub level: ErrorLevel,
}

/// Error severity level for display in ErrorConsole
///
/// Ordered from least to most severe for filtering purposes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorLevel {
    /// Debug information (shown only in debug mode)
    Debug,
    /// Informational message
    Info,
    /// Warning (operation continues)
    Warning,
    /// Error (recoverable)
    #[default]
    Error,
    /// Critical error (may require restart)
    Critical,
}

impl From<Severity> for ErrorLevel {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Debug => ErrorLevel::Debug,
            Severity::Info => ErrorLevel::Info,
            Severity::Warn => ErrorLevel::Warning,
            Severity::Error => ErrorLevel::Error,
            Severity::Critical => ErrorLevel::Critical,
        }
    }
}

impl ErrorLevel {
    /// Check if this level should be shown in production
    pub fn is_production_visible(self) -> bool {
        self >= ErrorLevel::Info
    }
}

/// State for ErrorConsole (owned by parent)
pub struct ErrorConsoleState {
    errors: VecDeque<ErrorEntry>,
    max_entries: usize,
    auto_dismiss: Option<Duration>,
}

impl Default for ErrorConsoleState {
    fn default() -> Self {
        Self::new()
    }
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
            self.errors
                .retain(|e| now.duration_since(e.timestamp) < duration);
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
    /// Get colors for error level from Theme
    /// Returns (background_color, text_color, icon)
    fn level_colors(level: ErrorLevel, theme: &Theme) -> (Color32, Color32, &'static str) {
        let (log_color, icon) = match level {
            ErrorLevel::Debug => (theme.log_debug, icons::WRENCH),
            ErrorLevel::Info => (theme.log_info, icons::INFO),
            ErrorLevel::Warning => (theme.log_warn, icons::WARNING),
            ErrorLevel::Error => (theme.log_error, icons::X_CIRCLE),
            ErrorLevel::Critical => (theme.log_critical, icons::FIRE),
        };

        // Background: semi-transparent version of the log color
        let bg_color = Color32::from_rgba_unmultiplied(
            log_color.r(),
            log_color.g(),
            log_color.b(),
            30, // Low alpha for subtle background
        );

        (bg_color, log_color, icon)
    }

    /// Get header color from theme
    fn header_color(theme: &Theme) -> Color32 {
        theme.log_error
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

        let theme = Theme::current(ctx.ui.ctx());

        // Collect dismiss clicks first
        let mut dismiss_index: Option<usize> = None;
        let mut clear_all = false;

        ctx.ui.vertical(|ui| {
            // Header with clear all button
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Errors ({})", state.len()))
                        .strong()
                        .color(Self::header_color(&theme)),
                );
                ui.add_space(8.0);
                if ui.small_button("Clear All").clicked() {
                    clear_all = true;
                }
            });

            ui.add_space(4.0);

            // Error list
            for (index, entry) in state.iter().enumerate() {
                let (bg_color, text_color, icon) = Self::level_colors(entry.level, &theme);

                egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(icon)
                                    .family(FontFamily::Name("icons".into()))
                                    .color(text_color),
                            );
                            ui.label(RichText::new(&entry.message).color(text_color));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("×").clicked() {
                                        dismiss_index = Some(index);
                                    }
                                },
                            );
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

        let theme = Theme::current(ui.ctx());
        let mut result = None;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("Errors ({})", state.len()))
                        .strong()
                        .color(Self::header_color(&theme)),
                );
                ui.add_space(8.0);
                if ui.small_button("Clear All").clicked() {
                    result = Some(ErrorConsoleMsg::DismissAll);
                }
            });

            ui.add_space(4.0);

            for (index, entry) in state.iter().enumerate() {
                let (bg_color, text_color, icon) = Self::level_colors(entry.level, &theme);

                egui::Frame::new()
                    .fill(bg_color)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(icon)
                                    .family(FontFamily::Name("icons".into()))
                                    .color(text_color),
                            );
                            ui.label(RichText::new(&entry.message).color(text_color));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("×").clicked() && result.is_none() {
                                        result = Some(ErrorConsoleMsg::Dismiss(index));
                                    }
                                },
                            );
                        });
                    });

                ui.add_space(2.0);
            }
        });

        result
    }
}
