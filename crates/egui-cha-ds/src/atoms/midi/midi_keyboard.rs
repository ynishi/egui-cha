//! MidiKeyboard - Piano keyboard display with note visualization
//!
//! A piano keyboard for displaying MIDI input, note triggers, and velocity.
//!
//! # Example
//! ```ignore
//! MidiKeyboard::new()
//!     .octaves(2)
//!     .start_octave(3)
//!     .active_notes(&model.pressed_notes)  // Vec<(note, velocity)>
//!     .show_with(ctx, |event| match event {
//!         KeyboardEvent::NoteOn(note, vel) => Msg::NoteOn(note, vel),
//!         KeyboardEvent::NoteOff(note) => Msg::NoteOff(note),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Keyboard events
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyboardEvent {
    /// Note on (note number 0-127, velocity 0-127)
    NoteOn(u8, u8),
    /// Note off (note number)
    NoteOff(u8),
}

/// Active note with velocity
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveNote {
    /// MIDI note number (0-127)
    pub note: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Optional color override
    pub color: Option<Color32>,
}

impl ActiveNote {
    /// Create from note and velocity
    pub fn new(note: u8, velocity: u8) -> Self {
        Self {
            note,
            velocity,
            color: None,
        }
    }

    /// With custom color
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl From<(u8, u8)> for ActiveNote {
    fn from((note, velocity): (u8, u8)) -> Self {
        Self::new(note, velocity)
    }
}

/// Piano keyboard display
pub struct MidiKeyboard<'a> {
    octaves: u8,
    start_octave: i8,
    active_notes: &'a [ActiveNote],
    white_key_width: f32,
    white_key_height: f32,
    show_labels: bool,
    show_velocity: bool,
    clickable: bool,
    highlight_color: Option<Color32>,
}

impl<'a> MidiKeyboard<'a> {
    /// Create a new keyboard
    pub fn new() -> Self {
        Self {
            octaves: 2,
            start_octave: 4,
            active_notes: &[],
            white_key_width: 24.0,
            white_key_height: 80.0,
            show_labels: true,
            show_velocity: true,
            clickable: true,
            highlight_color: None,
        }
    }

    /// Set number of octaves to display
    pub fn octaves(mut self, octaves: u8) -> Self {
        self.octaves = octaves.clamp(1, 10);
        self
    }

    /// Set starting octave (-2 to 8)
    pub fn start_octave(mut self, octave: i8) -> Self {
        self.start_octave = octave.clamp(-2, 8);
        self
    }

    /// Set active (pressed) notes
    pub fn active_notes(mut self, notes: &'a [ActiveNote]) -> Self {
        self.active_notes = notes;
        self
    }

    /// Set key size
    pub fn key_size(mut self, width: f32, height: f32) -> Self {
        self.white_key_width = width;
        self.white_key_height = height;
        self
    }

    /// Show/hide note labels (C4, D4, etc.)
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Show/hide velocity indicators
    pub fn show_velocity(mut self, show: bool) -> Self {
        self.show_velocity = show;
        self
    }

    /// Enable/disable click interaction
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Set highlight color for active notes
    pub fn highlight_color(mut self, color: Color32) -> Self {
        self.highlight_color = Some(color);
        self
    }

    /// TEA-style: Show keyboard and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(KeyboardEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show keyboard, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<KeyboardEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<KeyboardEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        // 7 white keys per octave
        let white_keys_per_octave = 7;
        let total_white_keys = self.octaves as usize * white_keys_per_octave;
        let total_width = total_white_keys as f32 * self.white_key_width;

        let black_key_width = self.white_key_width * 0.6;
        let black_key_height = self.white_key_height * 0.6;

        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(total_width, self.white_key_height),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Key patterns: C D E F G A B (white), C# D# F# G# A# (black)
        // Black key positions relative to white keys: after C, D, F, G, A
        let black_key_positions = [0, 1, 3, 4, 5]; // indices where black keys appear (after these white keys)

        // First pass: collect interaction info
        struct KeyInfo {
            rect: Rect,
            note: u8,
            is_black: bool,
            is_active: bool,
            velocity: u8,
            color: Option<Color32>,
        }

        let mut keys: Vec<KeyInfo> = Vec::new();

        // Generate all keys
        for octave in 0..self.octaves {
            let octave_offset = octave as usize * white_keys_per_octave;
            let midi_octave = (self.start_octave + octave as i8) as i32;

            // White keys
            for (i, note_in_octave) in [0, 2, 4, 5, 7, 9, 11].iter().enumerate() {
                let x = rect.min.x + (octave_offset + i) as f32 * self.white_key_width;
                let key_rect = Rect::from_min_size(
                    egui::pos2(x, rect.min.y),
                    Vec2::new(self.white_key_width, self.white_key_height),
                );

                let note = ((midi_octave + 1) * 12 + *note_in_octave as i32) as u8;
                let active_note = self.active_notes.iter().find(|n| n.note == note);

                keys.push(KeyInfo {
                    rect: key_rect,
                    note,
                    is_black: false,
                    is_active: active_note.is_some(),
                    velocity: active_note.map(|n| n.velocity).unwrap_or(0),
                    color: active_note.and_then(|n| n.color),
                });
            }

            // Black keys
            for (i, &white_idx) in black_key_positions.iter().enumerate() {
                let note_in_octave = match i {
                    0 => 1,  // C#
                    1 => 3,  // D#
                    2 => 6,  // F#
                    3 => 8,  // G#
                    4 => 10, // A#
                    _ => continue,
                };

                let x = rect.min.x
                    + (octave_offset + white_idx) as f32 * self.white_key_width
                    + self.white_key_width
                    - black_key_width / 2.0;

                let key_rect = Rect::from_min_size(
                    egui::pos2(x, rect.min.y),
                    Vec2::new(black_key_width, black_key_height),
                );

                let note = ((midi_octave + 1) * 12 + note_in_octave) as u8;
                let active_note = self.active_notes.iter().find(|n| n.note == note);

                keys.push(KeyInfo {
                    rect: key_rect,
                    note,
                    is_black: true,
                    is_active: active_note.is_some(),
                    velocity: active_note.map(|n| n.velocity).unwrap_or(0),
                    color: active_note.and_then(|n| n.color),
                });
            }
        }

        // Handle interactions (check black keys first as they're on top)
        if self.clickable {
            // Sort by is_black (black keys checked first)
            let mut interaction_order: Vec<usize> = (0..keys.len()).collect();
            interaction_order.sort_by(|&a, &b| keys[b].is_black.cmp(&keys[a].is_black));

            for &idx in &interaction_order {
                let key = &keys[idx];
                let response = ui.allocate_rect(key.rect, Sense::click());

                if response.clicked() {
                    if key.is_active {
                        event = Some(KeyboardEvent::NoteOff(key.note));
                    } else {
                        event = Some(KeyboardEvent::NoteOn(key.note, 100));
                    }
                    break;
                }
            }
        }

        // Second pass: draw all keys
        let painter = ui.painter();
        let highlight = self.highlight_color.unwrap_or(theme.primary);

        // Draw white keys first
        for key in keys.iter().filter(|k| !k.is_black) {
            let bg_color = if key.is_active {
                let vel_factor = key.velocity as f32 / 127.0;
                let c = key.color.unwrap_or(highlight);
                Color32::from_rgba_unmultiplied(
                    c.r(),
                    c.g(),
                    c.b(),
                    (100.0 + vel_factor * 155.0) as u8,
                )
            } else {
                Color32::from_rgb(250, 250, 250)
            };

            painter.rect_filled(key.rect, 2.0, bg_color);
            painter.rect_stroke(
                key.rect,
                2.0,
                Stroke::new(1.0, theme.border),
                egui::StrokeKind::Inside,
            );

            // Note label
            if self.show_labels {
                let note_name = Self::note_name(key.note);
                if note_name.starts_with('C') && !note_name.contains('#') {
                    painter.text(
                        egui::pos2(key.rect.center().x, key.rect.max.y - 12.0),
                        egui::Align2::CENTER_CENTER,
                        note_name,
                        egui::FontId::proportional(theme.font_size_xs * 0.8),
                        theme.text_muted,
                    );
                }
            }

            // Velocity bar
            if self.show_velocity && key.is_active {
                let vel_height = (key.velocity as f32 / 127.0) * 20.0;
                let vel_rect = Rect::from_min_size(
                    egui::pos2(key.rect.min.x + 2.0, key.rect.max.y - vel_height - 2.0),
                    Vec2::new(key.rect.width() - 4.0, vel_height),
                );
                let vel_color = key.color.unwrap_or(highlight);
                painter.rect_filled(vel_rect, 1.0, vel_color);
            }
        }

        // Draw black keys on top
        for key in keys.iter().filter(|k| k.is_black) {
            let bg_color = if key.is_active {
                let vel_factor = key.velocity as f32 / 127.0;
                let c = key.color.unwrap_or(highlight);
                Color32::from_rgba_unmultiplied(
                    c.r(),
                    c.g(),
                    c.b(),
                    (150.0 + vel_factor * 105.0) as u8,
                )
            } else {
                Color32::from_rgb(30, 30, 35)
            };

            painter.rect_filled(key.rect, 2.0, bg_color);

            // Velocity indicator for black keys
            if self.show_velocity && key.is_active {
                let vel_height = (key.velocity as f32 / 127.0) * 15.0;
                let vel_rect = Rect::from_min_size(
                    egui::pos2(key.rect.min.x + 2.0, key.rect.max.y - vel_height - 2.0),
                    Vec2::new(key.rect.width() - 4.0, vel_height),
                );
                let vel_color = key.color.unwrap_or(highlight);
                painter.rect_filled(vel_rect, 1.0, vel_color);
            }
        }

        event
    }

    fn note_name(note: u8) -> String {
        let names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let octave = (note as i32 / 12) - 1;
        let name_idx = (note % 12) as usize;
        format!("{}{}", names[name_idx], octave)
    }
}

impl Default for MidiKeyboard<'_> {
    fn default() -> Self {
        Self::new()
    }
}
