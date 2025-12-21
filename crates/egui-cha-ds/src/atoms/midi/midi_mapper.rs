//! MidiMapper atom - MIDI/OSC parameter mapping UI for VJ applications
//!
//! A component for assigning MIDI CC, notes, and OSC messages to parameters.
//! Supports learn mode, visual feedback, and mapping management.

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// MIDI message type for mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MidiMsgType {
    #[default]
    CC,
    Note,
    PitchBend,
    Aftertouch,
    ProgramChange,
}

impl MidiMsgType {
    pub fn label(&self) -> &'static str {
        match self {
            MidiMsgType::CC => "CC",
            MidiMsgType::Note => "Note",
            MidiMsgType::PitchBend => "Bend",
            MidiMsgType::Aftertouch => "AT",
            MidiMsgType::ProgramChange => "PC",
        }
    }

    pub fn all() -> &'static [MidiMsgType] {
        &[
            MidiMsgType::CC,
            MidiMsgType::Note,
            MidiMsgType::PitchBend,
            MidiMsgType::Aftertouch,
            MidiMsgType::ProgramChange,
        ]
    }
}

/// A parameter that can be mapped
#[derive(Debug, Clone)]
pub struct MappableParam {
    pub id: String,
    pub name: String,
    pub group: Option<String>,
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
}

impl MappableParam {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            group: None,
            current_value: 0.0,
            min_value: 0.0,
            max_value: 1.0,
        }
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.current_value = value;
        self
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }
}

/// A MIDI mapping assignment
#[derive(Debug, Clone, PartialEq)]
pub struct MidiMapping {
    pub param_id: String,
    pub msg_type: MidiMsgType,
    pub channel: u8,
    pub number: u8,
    pub min_out: f32,
    pub max_out: f32,
    pub inverted: bool,
}

impl MidiMapping {
    pub fn new(
        param_id: impl Into<String>,
        msg_type: MidiMsgType,
        channel: u8,
        number: u8,
    ) -> Self {
        Self {
            param_id: param_id.into(),
            msg_type,
            channel,
            number,
            min_out: 0.0,
            max_out: 1.0,
            inverted: false,
        }
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min_out = min;
        self.max_out = max;
        self
    }

    pub fn with_inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    pub fn label(&self) -> String {
        match self.msg_type {
            MidiMsgType::CC => format!("Ch{} CC{}", self.channel + 1, self.number),
            MidiMsgType::Note => format!("Ch{} {}", self.channel + 1, note_name(self.number)),
            MidiMsgType::PitchBend => format!("Ch{} Bend", self.channel + 1),
            MidiMsgType::Aftertouch => format!("Ch{} AT", self.channel + 1),
            MidiMsgType::ProgramChange => format!("Ch{} PC{}", self.channel + 1, self.number),
        }
    }
}

fn note_name(note: u8) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note / 12) as i32 - 1;
    let name = names[(note % 12) as usize];
    format!("{}{}", name, octave)
}

/// Learn mode state
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LearnState {
    #[default]
    Inactive,
    WaitingForMidi(String),
    Learned,
}

/// Events emitted by MidiMapper
#[derive(Debug, Clone)]
pub enum MidiMapperEvent {
    StartLearn(String),
    CancelLearn,
    AssignMapping(MidiMapping),
    RemoveMapping(String),
    SelectParam(String),
    SetMappingRange {
        param_id: String,
        min: f32,
        max: f32,
    },
    ToggleInvert(String),
}

/// MIDI mapper widget
pub struct MidiMapper<'a> {
    params: &'a [MappableParam],
    mappings: &'a [MidiMapping],
    learn_state: &'a LearnState,
    selected_param: Option<&'a str>,
    last_midi: Option<(MidiMsgType, u8, u8)>,
    size: Vec2,
    show_values: bool,
    show_ranges: bool,
    compact: bool,
}

impl<'a> MidiMapper<'a> {
    pub fn new(params: &'a [MappableParam], mappings: &'a [MidiMapping]) -> Self {
        Self {
            params,
            mappings,
            learn_state: &LearnState::Inactive,
            selected_param: None,
            last_midi: None,
            size: Vec2::new(350.0, 300.0),
            show_values: true,
            show_ranges: false,
            compact: false,
        }
    }

    pub fn learn_state(mut self, state: &'a LearnState) -> Self {
        self.learn_state = state;
        self
    }

    pub fn selected_param(mut self, id: Option<&'a str>) -> Self {
        self.selected_param = id;
        self
    }

    pub fn last_midi(mut self, msg: Option<(MidiMsgType, u8, u8)>) -> Self {
        self.last_midi = msg;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    pub fn show_ranges(mut self, show: bool) -> Self {
        self.show_ranges = show;
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(MidiMapperEvent) -> Msg,
    ) {
        if let Some(e) = self.show_internal(ctx.ui) {
            ctx.emit(on_event(e));
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<MidiMapperEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<MidiMapperEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<MidiMapperEvent> = None;

        let header_height = theme.spacing_xl;
        let row_height = if self.compact {
            theme.spacing_lg
        } else {
            theme.spacing_xl + theme.spacing_xs
        };

        let (rect, _response) = ui.allocate_exact_size(self.size, Sense::hover());

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let padding = theme.spacing_sm;
        let inner_rect = rect.shrink(padding);

        // Header area
        let header_rect =
            Rect::from_min_size(inner_rect.min, Vec2::new(inner_rect.width(), header_height));

        // Content area
        let content_rect = Rect::from_min_max(
            Pos2::new(inner_rect.min.x, inner_rect.min.y + header_height + padding),
            inner_rect.max,
        );

        // Collect interactions
        struct ParamRowInfo {
            param_id: String,
            rect: Rect,
            learn_rect: Rect,
            remove_rect: Option<Rect>,
            clicked: bool,
            learn_clicked: bool,
            remove_clicked: bool,
            hovered: bool,
        }

        let mut row_infos: Vec<ParamRowInfo> = Vec::new();
        let mut cancel_learn_clicked = false;

        // Header cancel button (when learning)
        if matches!(self.learn_state, LearnState::WaitingForMidi(_)) {
            let cancel_rect = Rect::from_min_size(
                Pos2::new(
                    header_rect.max.x - 60.0,
                    header_rect.min.y + (header_height - theme.spacing_md) / 2.0,
                ),
                Vec2::new(55.0, theme.spacing_md),
            );
            let resp = ui.allocate_rect(cancel_rect, Sense::click());
            cancel_learn_clicked = resp.clicked();
        }

        // Param rows
        let mut y = content_rect.min.y;
        for param in self.params.iter() {
            if y + row_height > content_rect.max.y {
                break;
            }

            let row_rect = Rect::from_min_size(
                Pos2::new(content_rect.min.x, y),
                Vec2::new(content_rect.width(), row_height),
            );

            // Learn button
            let learn_rect = Rect::from_min_size(
                Pos2::new(
                    row_rect.max.x - 50.0,
                    row_rect.min.y + (row_height - theme.spacing_md) / 2.0,
                ),
                Vec2::new(45.0, theme.spacing_md),
            );

            // Check if has mapping
            let has_mapping = self.mappings.iter().any(|m| m.param_id == param.id);

            // Remove button (if has mapping)
            let remove_rect = if has_mapping {
                Some(Rect::from_min_size(
                    Pos2::new(
                        learn_rect.min.x - 25.0,
                        row_rect.min.y + (row_height - theme.spacing_md) / 2.0,
                    ),
                    Vec2::new(20.0, theme.spacing_md),
                ))
            } else {
                None
            };

            let row_resp = ui.allocate_rect(row_rect, Sense::click());
            let learn_resp = ui.allocate_rect(learn_rect, Sense::click());
            let remove_resp = remove_rect.map(|r| ui.allocate_rect(r, Sense::click()));

            row_infos.push(ParamRowInfo {
                param_id: param.id.clone(),
                rect: row_rect,
                learn_rect,
                remove_rect,
                clicked: row_resp.clicked()
                    && !learn_resp.hovered()
                    && remove_resp.as_ref().map(|r| !r.hovered()).unwrap_or(true),
                learn_clicked: learn_resp.clicked(),
                remove_clicked: remove_resp.map(|r| r.clicked()).unwrap_or(false),
                hovered: row_resp.hovered(),
            });

            y += row_height;
        }

        // Drawing
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_md, theme.bg_secondary);

        // Header
        painter.rect_filled(header_rect, 0.0, theme.bg_tertiary);

        let header_text = match self.learn_state {
            LearnState::Inactive => "MIDI Mapper",
            LearnState::WaitingForMidi(_param_id) => {
                // Show which param we're learning for
                "Waiting for MIDI..."
            }
            LearnState::Learned => "Mapped!",
        };

        painter.text(
            Pos2::new(header_rect.min.x + theme.spacing_sm, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            header_text,
            egui::FontId::proportional(theme.font_size_sm),
            theme.text_primary,
        );

        // Last MIDI indicator
        if let Some((msg_type, ch, num)) = self.last_midi {
            let midi_text = match msg_type {
                MidiMsgType::CC => format!("Ch{} CC{}", ch + 1, num),
                MidiMsgType::Note => format!("Ch{} {}", ch + 1, note_name(num)),
                _ => format!("Ch{} {}", ch + 1, msg_type.label()),
            };
            painter.text(
                Pos2::new(header_rect.center().x, header_rect.center().y),
                egui::Align2::CENTER_CENTER,
                &midi_text,
                egui::FontId::proportional(theme.font_size_xs),
                theme.state_info,
            );
        }

        // Cancel button
        if matches!(self.learn_state, LearnState::WaitingForMidi(_)) {
            let cancel_rect = Rect::from_min_size(
                Pos2::new(
                    header_rect.max.x - 60.0,
                    header_rect.min.y + (header_height - theme.spacing_md) / 2.0,
                ),
                Vec2::new(55.0, theme.spacing_md),
            );
            painter.rect_filled(
                cancel_rect,
                theme.radius_sm,
                theme.state_danger.gamma_multiply(0.3),
            );
            painter.text(
                cancel_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Cancel",
                egui::FontId::proportional(theme.font_size_xs),
                theme.state_danger,
            );
        }

        // Param rows
        for (info, param) in row_infos.iter().zip(self.params.iter()) {
            let is_selected = self.selected_param == Some(&param.id);
            let is_learning =
                matches!(self.learn_state, LearnState::WaitingForMidi(ref id) if id == &param.id);
            let mapping = self.mappings.iter().find(|m| m.param_id == param.id);
            let is_hovered = info.hovered;

            // Row background
            let bg = if is_learning {
                theme.state_warning.gamma_multiply(0.2)
            } else if is_selected {
                theme.primary.gamma_multiply(0.2)
            } else if is_hovered {
                theme.bg_tertiary
            } else {
                Color32::TRANSPARENT
            };
            painter.rect_filled(info.rect, 0.0, bg);

            // Param name
            let name_x = info.rect.min.x + theme.spacing_sm;
            painter.text(
                Pos2::new(name_x, info.rect.min.y + row_height * 0.35),
                egui::Align2::LEFT_CENTER,
                &param.name,
                egui::FontId::proportional(theme.font_size_sm),
                theme.text_primary,
            );

            // Group label
            if let Some(ref group) = param.group {
                painter.text(
                    Pos2::new(name_x, info.rect.min.y + row_height * 0.7),
                    egui::Align2::LEFT_CENTER,
                    group,
                    egui::FontId::proportional(theme.font_size_xs * 0.9),
                    theme.text_muted,
                );
            }

            // Current value bar
            if self.show_values {
                let bar_width = 60.0;
                let bar_x = info.rect.min.x + 120.0;
                let bar_rect = Rect::from_min_size(
                    Pos2::new(bar_x, info.rect.center().y - 3.0),
                    Vec2::new(bar_width, 6.0),
                );
                painter.rect_filled(bar_rect, 3.0, theme.bg_tertiary);

                let normalized =
                    (param.current_value - param.min_value) / (param.max_value - param.min_value);
                let fill_rect = Rect::from_min_size(
                    bar_rect.min,
                    Vec2::new(bar_width * normalized.clamp(0.0, 1.0), 6.0),
                );
                painter.rect_filled(fill_rect, 3.0, theme.primary);

                // Value text
                painter.text(
                    Pos2::new(bar_rect.max.x + theme.spacing_xs, bar_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &format!("{:.0}%", normalized * 100.0),
                    egui::FontId::proportional(theme.font_size_xs * 0.9),
                    theme.text_muted,
                );
            }

            // Mapping info
            if let Some(m) = mapping {
                let mapping_x = info.learn_rect.min.x - 80.0;
                let mapping_color = if m.inverted {
                    theme.state_warning
                } else {
                    theme.state_success
                };
                painter.text(
                    Pos2::new(mapping_x, info.rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &m.label(),
                    egui::FontId::proportional(theme.font_size_xs),
                    mapping_color,
                );
            }

            // Remove button
            if let Some(remove_rect) = info.remove_rect {
                painter.rect_filled(
                    remove_rect,
                    theme.radius_sm,
                    theme.state_danger.gamma_multiply(0.3),
                );
                painter.text(
                    remove_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "×",
                    egui::FontId::proportional(theme.font_size_sm),
                    theme.state_danger,
                );
            }

            // Learn button
            let learn_bg = if is_learning {
                theme.state_warning
            } else if mapping.is_some() {
                theme.state_success.gamma_multiply(0.3)
            } else {
                theme.bg_tertiary
            };
            let learn_text = if is_learning {
                "..."
            } else if mapping.is_some() {
                "Edit"
            } else {
                "Learn"
            };
            let learn_color = if is_learning {
                theme.bg_primary
            } else {
                theme.text_secondary
            };

            painter.rect_filled(info.learn_rect, theme.radius_sm, learn_bg);
            painter.text(
                info.learn_rect.center(),
                egui::Align2::CENTER_CENTER,
                learn_text,
                egui::FontId::proportional(theme.font_size_xs),
                learn_color,
            );

            // Row separator
            painter.line_segment(
                [
                    Pos2::new(info.rect.min.x, info.rect.max.y),
                    Pos2::new(info.rect.max.x, info.rect.max.y),
                ],
                Stroke::new(0.5, theme.border.gamma_multiply(0.5)),
            );
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_md,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        // Process events
        if cancel_learn_clicked {
            event = Some(MidiMapperEvent::CancelLearn);
        }

        if event.is_none() {
            for info in row_infos.iter() {
                if info.remove_clicked {
                    event = Some(MidiMapperEvent::RemoveMapping(info.param_id.clone()));
                    break;
                }
                if info.learn_clicked {
                    event = Some(MidiMapperEvent::StartLearn(info.param_id.clone()));
                    break;
                }
                if info.clicked {
                    event = Some(MidiMapperEvent::SelectParam(info.param_id.clone()));
                    break;
                }
            }
        }

        event
    }
}
