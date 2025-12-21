//! MidiMonitor - MIDI activity and CC state display
//!
//! A monitor for displaying MIDI controller state, CC values, and recent activity.
//!
//! # Example
//! ```ignore
//! MidiMonitor::new()
//!     .device_name("Arturia KeyLab")
//!     .cc_values(&model.cc_state)  // HashMap<u8, u8> or &[(cc, value)]
//!     .recent_messages(&model.midi_log)
//!     .show(ui);
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// MIDI message types for display
#[derive(Clone, Debug, PartialEq)]
pub enum MidiMessage {
    /// Note On (channel, note, velocity)
    NoteOn(u8, u8, u8),
    /// Note Off (channel, note)
    NoteOff(u8, u8),
    /// Control Change (channel, cc, value)
    ControlChange(u8, u8, u8),
    /// Program Change (channel, program)
    ProgramChange(u8, u8),
    /// Pitch Bend (channel, value -8192 to 8191)
    PitchBend(u8, i16),
    /// Aftertouch (channel, pressure)
    Aftertouch(u8, u8),
    /// Clock
    Clock,
    /// Start
    Start,
    /// Stop
    Stop,
    /// Continue
    Continue,
}

impl MidiMessage {
    /// Format message for display
    pub fn format(&self) -> String {
        match self {
            MidiMessage::NoteOn(ch, note, vel) => {
                format!("NoteOn  ch:{} n:{:3} v:{:3}", ch + 1, note, vel)
            }
            MidiMessage::NoteOff(ch, note) => {
                format!("NoteOff ch:{} n:{:3}", ch + 1, note)
            }
            MidiMessage::ControlChange(ch, cc, val) => {
                format!("CC      ch:{} cc:{:3} v:{:3}", ch + 1, cc, val)
            }
            MidiMessage::ProgramChange(ch, prog) => {
                format!("PC      ch:{} p:{:3}", ch + 1, prog)
            }
            MidiMessage::PitchBend(ch, val) => {
                format!("Bend    ch:{} v:{:5}", ch + 1, val)
            }
            MidiMessage::Aftertouch(ch, pressure) => {
                format!("AT      ch:{} p:{:3}", ch + 1, pressure)
            }
            MidiMessage::Clock => "Clock".to_string(),
            MidiMessage::Start => "Start".to_string(),
            MidiMessage::Stop => "Stop".to_string(),
            MidiMessage::Continue => "Continue".to_string(),
        }
    }

    /// Get color for message type
    pub fn color(&self, theme: &Theme) -> Color32 {
        match self {
            MidiMessage::NoteOn(_, _, _) => theme.state_success,
            MidiMessage::NoteOff(_, _) => theme.text_muted,
            MidiMessage::ControlChange(_, _, _) => theme.primary,
            MidiMessage::ProgramChange(_, _) => theme.state_warning,
            MidiMessage::PitchBend(_, _) => Color32::from_rgb(200, 100, 200),
            MidiMessage::Aftertouch(_, _) => Color32::from_rgb(100, 200, 200),
            MidiMessage::Clock | MidiMessage::Start | MidiMessage::Stop | MidiMessage::Continue => {
                theme.text_secondary
            }
        }
    }
}

/// CC value with optional label
#[derive(Clone, Debug)]
pub struct CcValue {
    /// CC number (0-127)
    pub cc: u8,
    /// Current value (0-127)
    pub value: u8,
    /// Optional label
    pub label: Option<String>,
}

impl CcValue {
    /// Create from cc and value
    pub fn new(cc: u8, value: u8) -> Self {
        Self {
            cc,
            value,
            label: None,
        }
    }

    /// With label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl From<(u8, u8)> for CcValue {
    fn from((cc, value): (u8, u8)) -> Self {
        Self::new(cc, value)
    }
}

/// Monitor display mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MonitorMode {
    /// Show CC grid
    #[default]
    CcGrid,
    /// Show message log
    MessageLog,
    /// Show both (split view)
    Split,
}

/// MIDI monitor component
pub struct MidiMonitor<'a> {
    device_name: Option<&'a str>,
    cc_values: &'a [CcValue],
    messages: &'a [MidiMessage],
    mode: MonitorMode,
    width: f32,
    height: f32,
    cc_columns: usize,
    max_messages: usize,
    show_device_name: bool,
    connected: bool,
}

impl<'a> MidiMonitor<'a> {
    /// Create a new monitor
    pub fn new() -> Self {
        Self {
            device_name: None,
            cc_values: &[],
            messages: &[],
            mode: MonitorMode::default(),
            width: 300.0,
            height: 200.0,
            cc_columns: 4,
            max_messages: 10,
            show_device_name: true,
            connected: true,
        }
    }

    /// Set device name
    pub fn device_name(mut self, name: &'a str) -> Self {
        self.device_name = Some(name);
        self
    }

    /// Set CC values to display
    pub fn cc_values(mut self, values: &'a [CcValue]) -> Self {
        self.cc_values = values;
        self
    }

    /// Set recent messages
    pub fn messages(mut self, messages: &'a [MidiMessage]) -> Self {
        self.messages = messages;
        self
    }

    /// Set display mode
    pub fn mode(mut self, mode: MonitorMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set CC grid columns
    pub fn cc_columns(mut self, columns: usize) -> Self {
        self.cc_columns = columns.max(1);
        self
    }

    /// Set max messages to show
    pub fn max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    /// Show/hide device name
    pub fn show_device_name(mut self, show: bool) -> Self {
        self.show_device_name = show;
        self
    }

    /// Set connection status
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// Show the monitor (no events)
    pub fn show(self, ui: &mut Ui) {
        self.render(ui);
    }

    /// TEA-style show (for consistency, though no events)
    pub fn show_with<Msg>(self, ctx: &mut ViewCtx<'_, Msg>) {
        self.render(ctx.ui);
    }

    fn render(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());

        let header_height = if self.show_device_name { 24.0 } else { 0.0 };
        let content_height = self.height - header_height;

        let (rect, _) = ui.allocate_exact_size(Vec2::new(self.width, self.height), Sense::hover());

        if !ui.is_rect_visible(rect) {
            return;
        }

        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_sm, theme.bg_secondary);

        // Header with device name
        if self.show_device_name {
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(self.width, header_height));

            painter.rect_filled(header_rect, theme.radius_sm, theme.bg_tertiary);

            // Connection indicator
            let indicator_color = if self.connected {
                theme.state_success
            } else {
                theme.state_danger
            };
            painter.circle_filled(
                egui::pos2(rect.min.x + 12.0, header_rect.center().y),
                4.0,
                indicator_color,
            );

            // Device name
            let name = self.device_name.unwrap_or("No Device");
            painter.text(
                egui::pos2(rect.min.x + 24.0, header_rect.center().y),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(theme.font_size_sm),
                theme.text_primary,
            );

            // Mode indicator
            let mode_text = match self.mode {
                MonitorMode::CcGrid => "CC",
                MonitorMode::MessageLog => "LOG",
                MonitorMode::Split => "SPLIT",
            };
            painter.text(
                egui::pos2(rect.max.x - 8.0, header_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                mode_text,
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );
        }

        let content_rect = Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y + header_height),
            Vec2::new(self.width, content_height),
        );

        match self.mode {
            MonitorMode::CcGrid => {
                self.draw_cc_grid(painter, content_rect, &theme);
            }
            MonitorMode::MessageLog => {
                self.draw_message_log(painter, content_rect, &theme);
            }
            MonitorMode::Split => {
                let half_width = self.width / 2.0 - 2.0;
                let left_rect =
                    Rect::from_min_size(content_rect.min, Vec2::new(half_width, content_height));
                let right_rect = Rect::from_min_size(
                    egui::pos2(content_rect.min.x + half_width + 4.0, content_rect.min.y),
                    Vec2::new(half_width, content_height),
                );

                self.draw_cc_grid(painter, left_rect, &theme);
                self.draw_message_log(painter, right_rect, &theme);
            }
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );
    }

    fn draw_cc_grid(&self, painter: &egui::Painter, rect: Rect, theme: &Theme) {
        if self.cc_values.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No CC activity",
                egui::FontId::proportional(theme.font_size_sm),
                theme.text_muted,
            );
            return;
        }

        let padding = 4.0;
        let cell_width = (rect.width() - padding * 2.0) / self.cc_columns as f32;
        let cell_height = 32.0;

        let rows = (self.cc_values.len() + self.cc_columns - 1) / self.cc_columns;
        let start_y = rect.min.y + padding;

        for (idx, cc) in self.cc_values.iter().enumerate() {
            let col = idx % self.cc_columns;
            let row = idx / self.cc_columns;

            if row >= rows {
                break;
            }

            let x = rect.min.x + padding + col as f32 * cell_width;
            let y = start_y + row as f32 * cell_height;

            let cell_rect = Rect::from_min_size(
                egui::pos2(x, y),
                Vec2::new(cell_width - 2.0, cell_height - 2.0),
            );

            // Cell background
            painter.rect_filled(cell_rect, 2.0, theme.bg_primary);

            // CC label
            let default_label = format!("CC{}", cc.cc);
            let label = cc.label.as_deref().unwrap_or(&default_label);
            painter.text(
                egui::pos2(cell_rect.min.x + 4.0, cell_rect.min.y + 8.0),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(theme.font_size_xs * 0.9),
                theme.text_secondary,
            );

            // Value bar
            let bar_rect = Rect::from_min_size(
                egui::pos2(cell_rect.min.x + 4.0, cell_rect.max.y - 8.0),
                Vec2::new(cell_rect.width() - 8.0, 4.0),
            );
            painter.rect_filled(bar_rect, 1.0, theme.bg_tertiary);

            let fill_width = bar_rect.width() * (cc.value as f32 / 127.0);
            let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_width, 4.0));
            painter.rect_filled(fill_rect, 1.0, theme.primary);

            // Value text
            painter.text(
                egui::pos2(cell_rect.max.x - 4.0, cell_rect.min.y + 8.0),
                egui::Align2::RIGHT_CENTER,
                format!("{}", cc.value),
                egui::FontId::monospace(theme.font_size_xs),
                theme.text_primary,
            );
        }
    }

    fn draw_message_log(&self, painter: &egui::Painter, rect: Rect, theme: &Theme) {
        if self.messages.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No MIDI activity",
                egui::FontId::proportional(theme.font_size_sm),
                theme.text_muted,
            );
            return;
        }

        let padding = 4.0;
        let line_height = 16.0;
        let start_y = rect.min.y + padding;

        let messages_to_show = self.messages.len().min(self.max_messages);
        let start_idx = self.messages.len().saturating_sub(self.max_messages);

        for (i, msg) in self.messages[start_idx..].iter().enumerate() {
            if i >= messages_to_show {
                break;
            }

            let y = start_y + i as f32 * line_height;
            if y + line_height > rect.max.y {
                break;
            }

            let color = msg.color(theme);
            let text = msg.format();

            painter.text(
                egui::pos2(rect.min.x + padding, y + line_height / 2.0),
                egui::Align2::LEFT_CENTER,
                text,
                egui::FontId::monospace(theme.font_size_xs),
                color,
            );
        }
    }
}

impl Default for MidiMonitor<'_> {
    fn default() -> Self {
        Self::new()
    }
}
