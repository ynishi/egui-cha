//! Severity-based log display components
//!
//! Uses Theme's log_* colors for consistent severity visualization.

use crate::atoms::icons;
use crate::Theme;
use egui::{Color32, FontFamily, RichText, Ui};

/// Log severity level
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogSeverity {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Critical,
}

impl LogSeverity {
    /// Get the icon for this severity level
    pub fn icon(self) -> &'static str {
        match self {
            LogSeverity::Debug => icons::WRENCH,
            LogSeverity::Info => icons::INFO,
            LogSeverity::Warn => icons::WARNING,
            LogSeverity::Error => icons::X_CIRCLE,
            LogSeverity::Critical => icons::FIRE,
        }
    }

    /// Get the color from theme
    pub fn color(self, theme: &Theme) -> Color32 {
        match self {
            LogSeverity::Debug => theme.log_debug,
            LogSeverity::Info => theme.log_info,
            LogSeverity::Warn => theme.log_warn,
            LogSeverity::Error => theme.log_error,
            LogSeverity::Critical => theme.log_critical,
        }
    }

    /// Get a semi-transparent background color
    pub fn bg_color(self, theme: &Theme) -> Color32 {
        let c = self.color(theme);
        Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 30)
    }

    /// Get label text
    pub fn label(self) -> &'static str {
        match self {
            LogSeverity::Debug => "DEBUG",
            LogSeverity::Info => "INFO",
            LogSeverity::Warn => "WARN",
            LogSeverity::Error => "ERROR",
            LogSeverity::Critical => "CRITICAL",
        }
    }
}

/// Severity log text component
pub struct SeverityLog {
    severity: LogSeverity,
    message: String,
    show_icon: bool,
    show_label: bool,
}

impl SeverityLog {
    /// Create a new severity log with message
    pub fn new(severity: LogSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            show_icon: true,
            show_label: false,
        }
    }

    /// Create a debug log
    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(LogSeverity::Debug, message)
    }

    /// Create an info log
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(LogSeverity::Info, message)
    }

    /// Create a warning log
    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(LogSeverity::Warn, message)
    }

    /// Create an error log
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(LogSeverity::Error, message)
    }

    /// Create a critical log
    pub fn critical(message: impl Into<String>) -> Self {
        Self::new(LogSeverity::Critical, message)
    }

    /// Show/hide the severity icon
    pub fn with_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }

    /// Show/hide the severity label (DEBUG, INFO, etc.)
    pub fn with_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Display the log entry
    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let color = self.severity.color(&theme);

        ui.horizontal(|ui| {
            if self.show_icon {
                ui.label(
                    RichText::new(self.severity.icon())
                        .family(FontFamily::Name("icons".into()))
                        .color(color),
                );
            }

            if self.show_label {
                ui.label(RichText::new(self.severity.label()).strong().color(color));
            }

            ui.label(RichText::new(&self.message).color(color));
        });
    }

    /// Display in a framed box with background
    pub fn show_framed(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());
        let color = self.severity.color(&theme);
        let bg_color = self.severity.bg_color(&theme);

        egui::Frame::new()
            .fill(bg_color)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.show_icon {
                        ui.label(
                            RichText::new(self.severity.icon())
                                .family(FontFamily::Name("icons".into()))
                                .color(color),
                        );
                    }

                    if self.show_label {
                        ui.label(RichText::new(self.severity.label()).strong().color(color));
                    }

                    ui.label(RichText::new(&self.message).color(color));
                });
            });
    }
}
