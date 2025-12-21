//! PianoRoll - MIDI note editor
//!
//! A piano roll for editing MIDI notes with keyboard, grid, and note manipulation.
//!
//! # Example
//! ```ignore
//! PianoRoll::new()
//!     .notes(&model.notes)
//!     .position(model.playhead)
//!     .bars(4)
//!     .show_with(ctx, |event| match event {
//!         PianoRollEvent::NoteAdd(note, start, dur) => Msg::AddNote(note, start, dur),
//!         PianoRollEvent::NoteMove(idx, note, start) => Msg::MoveNote(idx, note, start),
//!         PianoRollEvent::NoteResize(idx, dur) => Msg::ResizeNote(idx, dur),
//!         PianoRollEvent::NoteDelete(idx) => Msg::DeleteNote(idx),
//!         PianoRollEvent::Seek(pos) => Msg::Seek(pos),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Piano roll events
#[derive(Clone, Debug, PartialEq)]
pub enum PianoRollEvent {
    /// Note added (midi_note, start_beat, duration_beats)
    NoteAdd(u8, f32, f32),
    /// Note moved (index, new_note, new_start)
    NoteMove(usize, u8, f32),
    /// Note resized (index, new_duration)
    NoteResize(usize, f32),
    /// Note deleted
    NoteDelete(usize),
    /// Note selected
    NoteSelect(usize),
    /// Seek to position (beats)
    Seek(f32),
}

/// A MIDI note in the piano roll
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MidiNote {
    /// MIDI note number (0-127)
    pub note: u8,
    /// Start position in beats
    pub start: f32,
    /// Duration in beats
    pub duration: f32,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Note color (optional override)
    pub color: Option<Color32>,
}

impl MidiNote {
    /// Create a new MIDI note
    pub fn new(note: u8, start: f32, duration: f32) -> Self {
        Self {
            note,
            start,
            duration,
            velocity: 100,
            color: None,
        }
    }

    /// Set velocity
    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity.min(127);
        self
    }

    /// Set custom color
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

/// Piano roll component
pub struct PianoRoll<'a> {
    notes: &'a [MidiNote],
    position: f32,
    bars: u32,
    beats_per_bar: u32,
    note_range: (u8, u8),
    selected: Option<usize>,
    snap_division: u32,
    key_width: f32,
    row_height: f32,
    show_velocity: bool,
    show_note_names: bool,
    editable: bool,
}

impl<'a> PianoRoll<'a> {
    /// Create a new piano roll
    pub fn new() -> Self {
        Self {
            notes: &[],
            position: 0.0,
            bars: 4,
            beats_per_bar: 4,
            note_range: (36, 84), // C2 to C6
            selected: None,
            snap_division: 4, // 1/4 note
            key_width: 40.0,
            row_height: 12.0,
            show_velocity: true,
            show_note_names: true,
            editable: true,
        }
    }

    /// Set notes
    pub fn notes(mut self, notes: &'a [MidiNote]) -> Self {
        self.notes = notes;
        self
    }

    /// Set playhead position (in beats)
    pub fn position(mut self, pos: f32) -> Self {
        self.position = pos.max(0.0);
        self
    }

    /// Set number of bars
    pub fn bars(mut self, bars: u32) -> Self {
        self.bars = bars.max(1);
        self
    }

    /// Set time signature
    pub fn time_signature(mut self, beats_per_bar: u32) -> Self {
        self.beats_per_bar = beats_per_bar.max(1);
        self
    }

    /// Set note range (min_note, max_note)
    pub fn note_range(mut self, min: u8, max: u8) -> Self {
        self.note_range = (min.min(127), max.min(127));
        self
    }

    /// Set selected note
    pub fn selected(mut self, idx: Option<usize>) -> Self {
        self.selected = idx;
        self
    }

    /// Set snap division (4 = quarter note, 8 = eighth, 16 = sixteenth)
    pub fn snap_division(mut self, div: u32) -> Self {
        self.snap_division = div.max(1);
        self
    }

    /// Set keyboard width
    pub fn key_width(mut self, width: f32) -> Self {
        self.key_width = width;
        self
    }

    /// Set row height
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Show velocity on notes
    pub fn show_velocity(mut self, show: bool) -> Self {
        self.show_velocity = show;
        self
    }

    /// Show note names on keyboard
    pub fn show_note_names(mut self, show: bool) -> Self {
        self.show_note_names = show;
        self
    }

    /// Enable/disable editing
    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// TEA-style: Show piano roll and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(PianoRollEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show piano roll, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<PianoRollEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<PianoRollEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        let num_notes = (self.note_range.1 - self.note_range.0 + 1) as usize;
        let total_beats = self.bars * self.beats_per_bar;
        let grid_width = ui.available_width() - self.key_width;
        let grid_height = num_notes as f32 * self.row_height;
        let total_height = grid_height;

        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), total_height),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let key_rect = Rect::from_min_size(rect.min, Vec2::new(self.key_width, grid_height));
        let grid_rect = Rect::from_min_size(
            egui::pos2(rect.min.x + self.key_width, rect.min.y),
            Vec2::new(grid_width, grid_height),
        );

        let beat_width = grid_width / total_beats as f32;
        let _snap_width = beat_width / (self.snap_division as f32 / self.beats_per_bar as f32);

        // Collect interactions first
        struct NoteInfo {
            rect: Rect,
            is_selected: bool,
            is_hovered: bool,
            velocity_alpha: u8,
            color: Color32,
        }

        let mut note_infos = Vec::new();

        for (idx, note) in self.notes.iter().enumerate() {
            if note.note < self.note_range.0 || note.note > self.note_range.1 {
                continue;
            }

            let row = (self.note_range.1 - note.note) as f32;
            let note_rect = Rect::from_min_size(
                egui::pos2(
                    grid_rect.min.x + note.start * beat_width,
                    grid_rect.min.y + row * self.row_height,
                ),
                Vec2::new(note.duration * beat_width, self.row_height - 1.0),
            );

            if note_rect.max.x < grid_rect.min.x || note_rect.min.x > grid_rect.max.x {
                continue;
            }

            let clipped_rect = Rect::from_min_max(
                egui::pos2(note_rect.min.x.max(grid_rect.min.x), note_rect.min.y),
                egui::pos2(note_rect.max.x.min(grid_rect.max.x), note_rect.max.y),
            );

            let response = ui.allocate_rect(clipped_rect, Sense::click_and_drag());
            let is_selected = self.selected == Some(idx);
            let is_hovered = response.hovered() || response.dragged();

            // Handle interactions
            if response.clicked() {
                event = Some(PianoRollEvent::NoteSelect(idx));
            }

            if response.dragged() && self.editable {
                let delta = response.drag_delta();
                let beat_delta = delta.x / beat_width;
                let note_delta = -(delta.y / self.row_height).round() as i8;

                let new_start = (note.start + beat_delta).max(0.0);
                let new_note = (note.note as i16 + note_delta as i16).clamp(0, 127) as u8;

                // Snap to grid
                let snapped_start =
                    (new_start * self.snap_division as f32).round() / self.snap_division as f32;

                if beat_delta.abs() > note_delta.abs() as f32 * beat_width / self.row_height {
                    event = Some(PianoRollEvent::NoteMove(idx, note.note, snapped_start));
                } else if note_delta != 0 {
                    event = Some(PianoRollEvent::NoteMove(idx, new_note, note.start));
                }
            }

            if response.secondary_clicked() && self.editable {
                event = Some(PianoRollEvent::NoteDelete(idx));
            }

            let base_color = note.color.unwrap_or(theme.primary);
            let velocity_alpha = if self.show_velocity {
                (note.velocity as f32 / 127.0 * 155.0 + 100.0) as u8
            } else {
                255
            };

            note_infos.push(NoteInfo {
                rect: clipped_rect,
                is_selected,
                is_hovered,
                velocity_alpha,
                color: base_color,
            });
        }

        // Grid click to add note
        let grid_response = ui.allocate_rect(grid_rect, Sense::click());
        if grid_response.double_clicked() && self.editable {
            if let Some(pos) = grid_response.interact_pointer_pos() {
                let beat = (pos.x - grid_rect.min.x) / beat_width;
                let row = (pos.y - grid_rect.min.y) / self.row_height;
                let note = self.note_range.1 - row as u8;

                // Snap
                let snapped_beat =
                    (beat * self.snap_division as f32).floor() / self.snap_division as f32;
                let default_duration =
                    1.0 / (self.snap_division as f32 / self.beats_per_bar as f32);

                if note >= self.note_range.0 && note <= self.note_range.1 {
                    event = Some(PianoRollEvent::NoteAdd(
                        note,
                        snapped_beat,
                        default_duration,
                    ));
                }
            }
        }

        // Click to seek
        if grid_response.clicked() && !grid_response.double_clicked() {
            if let Some(pos) = grid_response.interact_pointer_pos() {
                let beat = (pos.x - grid_rect.min.x) / beat_width;
                event = Some(PianoRollEvent::Seek(beat.max(0.0)));
            }
        }

        // Now paint everything
        let painter = ui.painter();

        // Keyboard background
        painter.rect_filled(key_rect, 0.0, theme.bg_secondary);

        // Draw piano keys
        for i in 0..num_notes {
            let note_num = self.note_range.1 - i as u8;
            let is_black = matches!(note_num % 12, 1 | 3 | 6 | 8 | 10);
            let y = key_rect.min.y + i as f32 * self.row_height;

            let key_color = if is_black {
                Color32::from_rgb(30, 30, 30)
            } else {
                Color32::from_rgb(240, 240, 240)
            };

            let key_rect_row = Rect::from_min_size(
                egui::pos2(key_rect.min.x, y),
                Vec2::new(self.key_width, self.row_height),
            );

            painter.rect_filled(key_rect_row, 0.0, key_color);

            // Note name on C notes
            if self.show_note_names && note_num % 12 == 0 {
                let octave = note_num / 12 - 1;
                let label = format!("C{}", octave);
                painter.text(
                    egui::pos2(key_rect.min.x + 4.0, y + self.row_height / 2.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(theme.font_size_xs * 0.8),
                    if is_black {
                        Color32::WHITE
                    } else {
                        Color32::BLACK
                    },
                );
            }

            // Key separator
            painter.line_segment(
                [egui::pos2(key_rect.min.x, y), egui::pos2(key_rect.max.x, y)],
                Stroke::new(0.5, theme.border),
            );
        }

        // Grid background
        painter.rect_filled(grid_rect, 0.0, theme.bg_primary);

        // Draw grid rows (note lanes)
        for i in 0..=num_notes {
            let y = grid_rect.min.y + i as f32 * self.row_height;
            let note_num = if i < num_notes {
                self.note_range.1 - i as u8
            } else {
                0
            };
            let is_black = matches!(note_num % 12, 1 | 3 | 6 | 8 | 10);

            // Row background for black keys
            if i < num_notes && is_black {
                let row_rect = Rect::from_min_size(
                    egui::pos2(grid_rect.min.x, y),
                    Vec2::new(grid_width, self.row_height),
                );
                painter.rect_filled(row_rect, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 20));
            }

            // Horizontal line
            let line_alpha = if note_num % 12 == 0 { 80 } else { 30 };
            painter.line_segment(
                [
                    egui::pos2(grid_rect.min.x, y),
                    egui::pos2(grid_rect.max.x, y),
                ],
                Stroke::new(
                    0.5,
                    Color32::from_rgba_unmultiplied(
                        theme.border.r(),
                        theme.border.g(),
                        theme.border.b(),
                        line_alpha,
                    ),
                ),
            );
        }

        // Draw grid columns (beats)
        for i in 0..=total_beats * self.snap_division / self.beats_per_bar {
            let beat = i as f32 / (self.snap_division as f32 / self.beats_per_bar as f32);
            let x = grid_rect.min.x + beat * beat_width;

            let is_bar = i % (self.snap_division * self.bars / self.beats_per_bar) == 0
                && i % self.snap_division == 0;
            let is_beat = i % (self.snap_division / self.beats_per_bar) == 0;

            let (stroke_width, alpha) = if is_bar {
                (1.5, 120)
            } else if is_beat {
                (1.0, 60)
            } else {
                (0.5, 25)
            };

            painter.line_segment(
                [
                    egui::pos2(x, grid_rect.min.y),
                    egui::pos2(x, grid_rect.max.y),
                ],
                Stroke::new(
                    stroke_width,
                    Color32::from_rgba_unmultiplied(
                        theme.border.r(),
                        theme.border.g(),
                        theme.border.b(),
                        alpha,
                    ),
                ),
            );

            // Bar numbers
            if is_bar && x > grid_rect.min.x {
                let bar_num = (beat / self.beats_per_bar as f32) as u32 + 1;
                painter.text(
                    egui::pos2(x + 2.0, grid_rect.min.y + 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("{}", bar_num),
                    egui::FontId::proportional(theme.font_size_xs * 0.7),
                    theme.text_muted,
                );
            }
        }

        // Draw notes
        for info in &note_infos {
            let note_color = if info.is_selected {
                Color32::WHITE
            } else if info.is_hovered {
                Color32::from_rgba_unmultiplied(info.color.r(), info.color.g(), info.color.b(), 255)
            } else {
                Color32::from_rgba_unmultiplied(
                    info.color.r(),
                    info.color.g(),
                    info.color.b(),
                    info.velocity_alpha,
                )
            };

            painter.rect_filled(info.rect, 2.0, note_color);

            if info.is_selected {
                painter.rect_stroke(
                    info.rect,
                    2.0,
                    Stroke::new(2.0, theme.primary),
                    egui::StrokeKind::Outside,
                );
            }
        }

        // Draw playhead
        let playhead_x = grid_rect.min.x + self.position * beat_width;
        if playhead_x >= grid_rect.min.x && playhead_x <= grid_rect.max.x {
            painter.line_segment(
                [
                    egui::pos2(playhead_x, grid_rect.min.y),
                    egui::pos2(playhead_x, grid_rect.max.y),
                ],
                Stroke::new(2.0, theme.state_success),
            );

            // Playhead triangle
            let tri_size = 6.0;
            let tri_points = vec![
                egui::pos2(playhead_x - tri_size, grid_rect.min.y),
                egui::pos2(playhead_x + tri_size, grid_rect.min.y),
                egui::pos2(playhead_x, grid_rect.min.y + tri_size),
            ];
            painter.add(egui::Shape::convex_polygon(
                tri_points,
                theme.state_success,
                Stroke::NONE,
            ));
        }

        // Border between keyboard and grid
        painter.line_segment(
            [
                egui::pos2(key_rect.max.x, rect.min.y),
                egui::pos2(key_rect.max.x, rect.max.y),
            ],
            Stroke::new(1.0, theme.border),
        );

        event
    }
}

impl Default for PianoRoll<'_> {
    fn default() -> Self {
        Self::new()
    }
}
