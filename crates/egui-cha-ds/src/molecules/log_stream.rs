//! LogStream - Real-time log viewer component
//!
//! A scrollable log viewer with filtering and auto-scroll capabilities.
//!
//! # Example
//!
//! ```ignore
//! // State initialization (in Model)
//! let mut log_state = LogStreamState::new();
//!
//! // Pushing logs
//! log_state.push_info("Brain #0", "Starting inference task");
//! log_state.push_warn("Manager", "Queue depth > 100");
//! log_state.push_error("Worker #42", "Connection timeout");
//!
//! // Rendering
//! LogStream::new(&log_state)
//!     .height(300.0)
//!     .show_toolbar(true)
//!     .show(ui);
//! ```

use crate::atoms::{Button, Input};
use crate::semantics::LogSeverity;
use crate::Theme;
use egui::{FontFamily, Response, RichText, ScrollArea, Ui};
use std::collections::VecDeque;
use std::time::Instant;

/// A single log entry
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Timestamp when the log was created
    pub timestamp: Instant,
    /// Severity level
    pub severity: LogSeverity,
    /// Source identifier (e.g., "Brain #0", "Worker #42")
    pub source: Option<String>,
    /// Log message
    pub message: String,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(severity: LogSeverity, message: impl Into<String>) -> Self {
        Self {
            timestamp: Instant::now(),
            severity,
            source: None,
            message: message.into(),
        }
    }

    /// Create with source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Filter settings for log display
#[derive(Clone, Debug, Default)]
pub struct LogFilter {
    /// Minimum severity to show (None = show all)
    pub min_severity: Option<LogSeverity>,
    /// Source filter (None = show all)
    pub source: Option<String>,
    /// Text search query
    pub search: String,
}

impl LogFilter {
    /// Check if an entry passes the filter
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Severity filter
        if let Some(min) = self.min_severity {
            if entry.severity < min {
                return false;
            }
        }

        // Source filter
        if let Some(ref src) = self.source {
            if let Some(ref entry_src) = entry.source {
                if !entry_src.contains(src) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Search filter
        if !self.search.is_empty() {
            let query = self.search.to_lowercase();
            let msg_match = entry.message.to_lowercase().contains(&query);
            let src_match = entry
                .source
                .as_ref()
                .map(|s| s.to_lowercase().contains(&query))
                .unwrap_or(false);
            if !msg_match && !src_match {
                return false;
            }
        }

        true
    }
}

/// State for LogStream (owned by parent)
pub struct LogStreamState {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    /// Auto-scroll to bottom on new entries
    pub auto_scroll: bool,
    /// Current filter settings
    pub filter: LogFilter,
    /// Track if new entries were added (for auto-scroll)
    scroll_to_bottom: bool,
}

impl Default for LogStreamState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogStreamState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 1000,
            auto_scroll: true,
            filter: LogFilter::default(),
            scroll_to_bottom: false,
        }
    }

    /// Set maximum entries to keep
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Push a log entry
    pub fn push(&mut self, entry: LogEntry) {
        self.entries.push_back(entry);

        // Trim old entries
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        if self.auto_scroll {
            self.scroll_to_bottom = true;
        }
    }

    /// Push a debug log
    pub fn push_debug(&mut self, source: &str, message: impl Into<String>) {
        self.push(LogEntry::new(LogSeverity::Debug, message).with_source(source));
    }

    /// Push an info log
    pub fn push_info(&mut self, source: &str, message: impl Into<String>) {
        self.push(LogEntry::new(LogSeverity::Info, message).with_source(source));
    }

    /// Push a warning log
    pub fn push_warn(&mut self, source: &str, message: impl Into<String>) {
        self.push(LogEntry::new(LogSeverity::Warn, message).with_source(source));
    }

    /// Push an error log
    pub fn push_error(&mut self, source: &str, message: impl Into<String>) {
        self.push(LogEntry::new(LogSeverity::Error, message).with_source(source));
    }

    /// Push a critical log
    pub fn push_critical(&mut self, source: &str, message: impl Into<String>) {
        self.push(LogEntry::new(LogSeverity::Critical, message).with_source(source));
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get total entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get filtered entries
    pub fn filtered_entries(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().filter(|e| self.filter.matches(e))
    }

    /// Get filtered entry count
    pub fn filtered_len(&self) -> usize {
        self.filtered_entries().count()
    }

    /// Check and consume scroll flag
    fn take_scroll_flag(&mut self) -> bool {
        std::mem::take(&mut self.scroll_to_bottom)
    }
}

/// Timestamp display format
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimestampFormat {
    /// Don't show timestamp
    None,
    /// Show time only (HH:MM:SS)
    #[default]
    TimeOnly,
    /// Show relative time (e.g., "2s ago")
    Relative,
}

/// LogStream component - displays a scrollable log view
pub struct LogStream<'a> {
    state: &'a mut LogStreamState,
    height: Option<f32>,
    show_timestamp: bool,
    show_source: bool,
    show_toolbar: bool,
    monospace: bool,
    timestamp_format: TimestampFormat,
}

impl<'a> LogStream<'a> {
    /// Create a new LogStream viewer
    pub fn new(state: &'a mut LogStreamState) -> Self {
        Self {
            state,
            height: None,
            show_timestamp: true,
            show_source: true,
            show_toolbar: true,
            monospace: true,
            timestamp_format: TimestampFormat::TimeOnly,
        }
    }

    /// Set fixed height (None = fill available space)
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    /// Show/hide timestamp column
    pub fn show_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    /// Show/hide source column
    pub fn show_source(mut self, show: bool) -> Self {
        self.show_source = show;
        self
    }

    /// Show/hide toolbar (filter controls)
    pub fn show_toolbar(mut self, show: bool) -> Self {
        self.show_toolbar = show;
        self
    }

    /// Use monospace font for log messages
    pub fn monospace(mut self, mono: bool) -> Self {
        self.monospace = mono;
        self
    }

    /// Set timestamp format
    pub fn timestamp_format(mut self, format: TimestampFormat) -> Self {
        self.timestamp_format = format;
        self
    }

    /// Show the log stream
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());

        let response = ui
            .vertical(|ui| {
                // Toolbar
                if self.show_toolbar {
                    self.render_toolbar(ui, &theme);
                    ui.add_space(theme.spacing_sm);
                }

                // Log entries
                self.render_entries(ui, &theme);
            })
            .response;

        response
    }

    fn render_toolbar(&mut self, ui: &mut Ui, theme: &Theme) {
        ui.horizontal(|ui| {
            // Severity filter dropdown
            let severity_options: [(_, Option<LogSeverity>); 6] = [
                ("All", None),
                ("Debug+", Some(LogSeverity::Debug)),
                ("Info+", Some(LogSeverity::Info)),
                ("Warn+", Some(LogSeverity::Warn)),
                ("Error+", Some(LogSeverity::Error)),
                ("Critical", Some(LogSeverity::Critical)),
            ];

            let current_label = severity_options
                .iter()
                .find(|(_, v)| *v == self.state.filter.min_severity)
                .map(|(l, _)| *l)
                .unwrap_or("All");

            egui::ComboBox::from_id_salt("log_severity_filter")
                .selected_text(current_label)
                .width(80.0)
                .show_ui(ui, |ui| {
                    for (label, severity) in &severity_options {
                        if ui
                            .selectable_label(self.state.filter.min_severity == *severity, *label)
                            .clicked()
                        {
                            self.state.filter.min_severity = *severity;
                        }
                    }
                });

            // Search input
            ui.add_space(theme.spacing_sm);
            Input::new()
                .placeholder("Search...")
                .desired_width(150.0)
                .show(ui, &mut self.state.filter.search);

            // Clear button
            ui.add_space(theme.spacing_sm);
            if Button::ghost("Clear").show(ui) {
                self.state.clear();
            }

            // Auto-scroll toggle
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.state.auto_scroll, "Auto-scroll");
            });
        });
    }

    fn render_entries(&mut self, ui: &mut Ui, theme: &Theme) {
        let scroll_to_bottom = self.state.take_scroll_flag();

        let scroll_area = if let Some(h) = self.height {
            ScrollArea::vertical().max_height(h)
        } else {
            ScrollArea::vertical()
        };

        scroll_area
            .auto_shrink([false, false])
            .stick_to_bottom(scroll_to_bottom)
            .show(ui, |ui| {
                let now = Instant::now();

                // Collect filtered entries (to avoid borrow issues)
                let entries: Vec<_> = self
                    .state
                    .entries
                    .iter()
                    .filter(|e| self.state.filter.matches(e))
                    .collect();

                if entries.is_empty() {
                    ui.label(
                        RichText::new("No log entries")
                            .italics()
                            .color(theme.text_muted),
                    );
                } else {
                    for entry in entries {
                        self.render_entry(ui, entry, theme, now);
                    }
                }
            });
    }

    fn render_entry(&self, ui: &mut Ui, entry: &LogEntry, theme: &Theme, now: Instant) {
        let severity_color = entry.severity.color(theme);

        ui.horizontal(|ui| {
            // Timestamp
            if self.show_timestamp && self.timestamp_format != TimestampFormat::None {
                let ts_text = match self.timestamp_format {
                    TimestampFormat::None => String::new(),
                    TimestampFormat::TimeOnly => {
                        let elapsed = now.duration_since(entry.timestamp);
                        let secs = elapsed.as_secs();
                        let mins = secs / 60;
                        let hours = mins / 60;
                        format!("{:02}:{:02}:{:02}", hours % 24, mins % 60, secs % 60)
                    }
                    TimestampFormat::Relative => {
                        let elapsed = now.duration_since(entry.timestamp);
                        if elapsed.as_secs() < 60 {
                            format!("{}s ago", elapsed.as_secs())
                        } else if elapsed.as_secs() < 3600 {
                            format!("{}m ago", elapsed.as_secs() / 60)
                        } else {
                            format!("{}h ago", elapsed.as_secs() / 3600)
                        }
                    }
                };
                ui.label(RichText::new(ts_text).color(theme.text_muted).monospace());
            }

            // Severity icon
            ui.label(
                RichText::new(entry.severity.icon())
                    .family(FontFamily::Name("icons".into()))
                    .color(severity_color),
            );

            // Severity label
            ui.label(
                RichText::new(format!("[{}]", entry.severity.label()))
                    .color(severity_color)
                    .strong(),
            );

            // Source
            if self.show_source {
                if let Some(ref source) = entry.source {
                    ui.label(RichText::new(source).color(theme.text_secondary));
                }
            }

            // Message
            let msg = if self.monospace {
                RichText::new(&entry.message).monospace()
            } else {
                RichText::new(&entry.message)
            };
            ui.label(msg);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogSeverity::Info, "Test message").with_source("TestSource");

        assert_eq!(entry.severity, LogSeverity::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.source, Some("TestSource".to_string()));
    }

    #[test]
    fn test_log_stream_state_push() {
        let mut state = LogStreamState::new().with_max_entries(5);

        for i in 0..10 {
            state.push_info("test", format!("Message {}", i));
        }

        // Should only keep last 5
        assert_eq!(state.len(), 5);
    }

    #[test]
    fn test_log_filter_severity() {
        let filter = LogFilter {
            min_severity: Some(LogSeverity::Warn),
            ..Default::default()
        };

        let debug = LogEntry::new(LogSeverity::Debug, "debug");
        let info = LogEntry::new(LogSeverity::Info, "info");
        let warn = LogEntry::new(LogSeverity::Warn, "warn");
        let error = LogEntry::new(LogSeverity::Error, "error");

        assert!(!filter.matches(&debug));
        assert!(!filter.matches(&info));
        assert!(filter.matches(&warn));
        assert!(filter.matches(&error));
    }

    #[test]
    fn test_log_filter_search() {
        let filter = LogFilter {
            search: "error".to_string(),
            ..Default::default()
        };

        let entry1 = LogEntry::new(LogSeverity::Info, "An error occurred");
        let entry2 = LogEntry::new(LogSeverity::Info, "All good");
        let entry3 = LogEntry::new(LogSeverity::Info, "ok").with_source("ErrorHandler");

        assert!(filter.matches(&entry1));
        assert!(!filter.matches(&entry2));
        assert!(filter.matches(&entry3)); // matches source
    }

    #[test]
    fn test_filtered_entries() {
        let mut state = LogStreamState::new();
        state.push_debug("src", "debug msg");
        state.push_info("src", "info msg");
        state.push_warn("src", "warn msg");
        state.push_error("src", "error msg");

        state.filter.min_severity = Some(LogSeverity::Warn);

        assert_eq!(state.filtered_len(), 2);
    }
}
