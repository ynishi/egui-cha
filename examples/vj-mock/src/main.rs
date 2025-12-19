//! VJ Software Mock - Live Area UI
//!
//! A mock VJ software UI replicating the Live Area from glitch-mix:
//! - Transport (BPM, Bar, Beat, Play/Rec, Source)
//! - Setup Selector
//! - Performance State (Playing/Queued, Switch, Quantize, Params)
//! - Phrase Grid
//! - Timing & Beat Sync (collapsible)
//! - Audio Reactivity (collapsible)
//! - MIDI Status (collapsible)

use egui::Color32;
use egui_cha::prelude::*;
use egui_cha_ds::icons;
use egui_cha_ds::prelude::*;

/// VJ software custom theme - deep black with cyan accents
fn vj_theme() -> Theme {
    Theme {
        variant: ThemeVariant::Dark,

        // Primary - Cyan accent (common in VJ/DJ software)
        primary: Color32::from_rgb(0, 200, 220),
        primary_hover: Color32::from_rgb(0, 180, 200),
        primary_text: Color32::BLACK,

        // Secondary - Muted cyan
        secondary: Color32::from_rgb(80, 120, 130),
        secondary_hover: Color32::from_rgb(60, 100, 110),
        secondary_text: Color32::WHITE,

        // Background - Deep black
        bg_primary: Color32::from_rgb(8, 8, 12),
        bg_secondary: Color32::from_rgb(16, 18, 24),
        bg_tertiary: Color32::from_rgb(28, 32, 40),

        // Text - High contrast
        text_primary: Color32::from_rgb(230, 235, 240),
        text_secondary: Color32::from_rgb(160, 170, 180),
        text_muted: Color32::from_rgb(100, 110, 120),

        // Semantic (background)
        success: Color32::from_rgb(40, 200, 100),
        warning: Color32::from_rgb(230, 180, 40),
        error: Color32::from_rgb(220, 60, 70),
        info: Color32::from_rgb(60, 160, 230),
        danger: Color32::from_rgb(220, 60, 70),

        // Semantic text
        success_text: Color32::BLACK,
        warning_text: Color32::BLACK,
        error_text: Color32::WHITE,
        info_text: Color32::BLACK,
        danger_text: Color32::WHITE,

        // Semantic hover
        success_hover: Color32::from_rgb(30, 180, 80),
        warning_hover: Color32::from_rgb(210, 160, 30),
        error_hover: Color32::from_rgb(200, 40, 50),
        info_hover: Color32::from_rgb(40, 140, 210),
        danger_hover: Color32::from_rgb(200, 40, 50),

        // Border - Subtle
        border: Color32::from_rgb(50, 55, 65),
        border_focus: Color32::from_rgb(0, 200, 220),

        // Spacing
        spacing_xs: 4.0,
        spacing_sm: 8.0,
        spacing_md: 16.0,
        spacing_lg: 24.0,
        spacing_xl: 32.0,

        // Radius - Minimal for professional look
        radius_sm: 2.0,
        radius_md: 4.0,
        radius_lg: 6.0,

        // Typography
        font_size_xs: 10.0,
        font_size_sm: 12.0,
        font_size_md: 14.0,
        font_size_lg: 16.0,
        font_size_xl: 20.0,
        font_size_2xl: 24.0,
        font_size_3xl: 30.0,
        line_height: 1.4,
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    egui_cha::run::<VjApp>(RunConfig::new("VJ Mock").with_size(1200.0, 900.0))
}

// ============================================================
// App
// ============================================================

struct VjApp;

#[derive(Clone, Debug)]
enum Msg {
    // Transport
    TogglePlay,
    ToggleRecord,
    SetBpm(f32),
    ToggleManualBpm,
    Tick,

    // Setup
    SelectSetup(usize),

    // Performance State
    SetSwitchMode(SwitchMode),
    SetQuantize(Quantize),
    SetParamMode(ParamMode),

    // Phrase Grid
    SelectPhrase(usize),
    QueuePhrase(usize),

    // Collapsible sections
    ToggleTimingSection,
    ToggleAudioSection,
    ToggleMidiSection,

    // Audio Reactivity
    ToggleAudioReactivity,

    // Lab Area
    SelectPatch(usize),
    NewPatch,
    DuplicatePatch,
    DeletePatch,
    SetPatchName(String),
    SetPatchAuthor(String),
    SetPatchDesc(String),
    SetPatchTags(String),
    SetLabViewMode(LabViewMode),

    // MODs
    SetModBlendMode(BlendMode),
    SetModOpacityA(f32),
    SetModOpacityB(f32),
    SetModBitDepth(f32),
    ToggleModEnabled,
    DeleteMod,

    // Textures
    AddImageTexture,
    AddVideoTexture,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SwitchMode {
    Auto,
    Manual,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Quantize {
    Immediate,
    Beat,
    Bar,
    TwoBars,
    FourBars,
}

impl Quantize {
    fn label(&self) -> &'static str {
        match self {
            Quantize::Immediate => "Immediate",
            Quantize::Beat => "Beat",
            Quantize::Bar => "Bar",
            Quantize::TwoBars => "2 Bars",
            Quantize::FourBars => "4 Bars",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ParamMode {
    Reset,
    Inherit,
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
enum TimingSource {
    None,
    AudioAnalysis,
    MidiClock,
    Manual,
}

impl TimingSource {
    fn label(&self) -> &'static str {
        match self {
            TimingSource::None => "None",
            TimingSource::AudioAnalysis => "Audio Analysis",
            TimingSource::MidiClock => "MIDI Clock",
            TimingSource::Manual => "Manual",
        }
    }
}

// ============================================================
// Lab Area Types
// ============================================================

#[derive(Clone, Copy, PartialEq, Debug)]
enum LabViewMode {
    Yaml,
    Basic,
    Pack,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum BlendMode {
    Normal,
    Add,
    Sub,
    Mul,
    Screen,
    Over,
    Diff,
    Xor,
    Or,
    And,
    Hard,
}

impl BlendMode {
    fn label(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Normal",
            BlendMode::Add => "Add",
            BlendMode::Sub => "Sub",
            BlendMode::Mul => "Mul",
            BlendMode::Screen => "Screen",
            BlendMode::Over => "Over",
            BlendMode::Diff => "Diff",
            BlendMode::Xor => "XOR",
            BlendMode::Or => "OR",
            BlendMode::And => "AND",
            BlendMode::Hard => "Hard",
        }
    }

    fn all() -> &'static [BlendMode] {
        &[
            BlendMode::Normal, BlendMode::Add, BlendMode::Sub, BlendMode::Mul,
            BlendMode::Screen, BlendMode::Over, BlendMode::Diff, BlendMode::Xor,
            BlendMode::Or, BlendMode::And, BlendMode::Hard,
        ]
    }
}

#[derive(Clone)]
struct Patch {
    name: String,
    author: String,
    desc: String,
    tags: String,
    version: String,
    created: u64,
}

impl Default for Patch {
    fn default() -> Self {
        Self {
            name: "patch-1".into(),
            author: "Unknown".into(),
            desc: String::new(),
            tags: String::new(),
            version: "v1.0.0".into(),
            created: 1766178516,
        }
    }
}

#[derive(Clone)]
struct ModEffect {
    name: String,
    effect_type: String,
    node_id: String,
    enabled: bool,
    blend_mode: BlendMode,
    opacity_a: f32,
    opacity_b: f32,
    bit_depth: f32,
}

impl Default for ModEffect {
    fn default() -> Self {
        Self {
            name: "Blend".into(),
            effect_type: "Effector".into(),
            node_id: "NodeId(1v3)".into(),
            enabled: true,
            blend_mode: BlendMode::Normal,
            opacity_a: 0.5,
            opacity_b: 0.5,
            bit_depth: 4.5,
        }
    }
}

#[derive(Clone)]
struct Texture {
    name: String,
    tex_type: TextureType,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TextureType {
    Image,
    Video,
}

struct Model {
    // Transport
    playing: bool,
    recording: bool,
    bpm: f32,
    bar: u32,
    beat: f32,
    timing_source: TimingSource,
    manual_bpm_enabled: bool,
    midi_status: bool,
    audio_status: bool,

    // Setup
    setups: Vec<Setup>,
    current_setup: usize,

    // Performance State
    current_phrase: Option<usize>,
    queued_phrase: Option<usize>,
    switch_mode: SwitchMode,
    quantize: Quantize,
    param_mode: ParamMode,

    // Collapsible sections
    timing_section_open: bool,
    audio_section_open: bool,
    midi_section_open: bool,

    // Audio Reactivity
    audio_reactivity_on: bool,
    audio_low: f32,
    audio_mid: f32,
    audio_high: f32,

    // Lab Area
    patches: Vec<Patch>,
    current_patch: usize,
    lab_view_mode: LabViewMode,
    mods: Vec<ModEffect>,
    textures: Vec<Texture>,
}

struct Setup {
    name: String,
    phrases: Vec<Phrase>,
}

#[derive(Clone)]
struct Phrase {
    name: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            playing: false,
            recording: false,
            bpm: 120.0,
            bar: 1,
            beat: 1.0,
            timing_source: TimingSource::None,
            manual_bpm_enabled: false,
            midi_status: false,
            audio_status: false,

            setups: vec![
                Setup {
                    name: "Setup 1".into(),
                    phrases: vec![
                        Phrase { name: "Phrase 1".into() },
                        Phrase { name: "Phrase 2".into() },
                    ],
                },
                Setup {
                    name: "Setup 2".into(),
                    phrases: vec![
                        Phrase { name: "Intro".into() },
                        Phrase { name: "Build".into() },
                        Phrase { name: "Drop".into() },
                    ],
                },
            ],
            current_setup: 0,

            current_phrase: None,
            queued_phrase: None,
            switch_mode: SwitchMode::Auto,
            quantize: Quantize::Bar,
            param_mode: ParamMode::Reset,

            timing_section_open: true,
            audio_section_open: true,
            midi_section_open: true,

            audio_reactivity_on: false,
            audio_low: 0.3,
            audio_mid: 0.5,
            audio_high: 0.7,

            // Lab Area
            patches: vec![
                Patch::default(),
                Patch {
                    name: "patch-1-copy".into(),
                    ..Patch::default()
                },
            ],
            current_patch: 0,
            lab_view_mode: LabViewMode::Basic,
            mods: vec![ModEffect::default()],
            textures: vec![],
        }
    }
}

impl App for VjApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        (Model::default(), Cmd::none())
    }

    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::TogglePlay => {
                model.playing = !model.playing;
                if model.playing {
                    return Cmd::delay(std::time::Duration::from_millis(50), Msg::Tick);
                }
            }
            Msg::ToggleRecord => {
                model.recording = !model.recording;
            }
            Msg::SetBpm(bpm) => {
                model.bpm = bpm;
            }
            Msg::ToggleManualBpm => {
                model.manual_bpm_enabled = !model.manual_bpm_enabled;
            }
            Msg::Tick => {
                if model.playing {
                    let beat_per_tick = model.bpm / 60.0 * 0.05;
                    model.beat += beat_per_tick;
                    if model.beat > 4.0 {
                        model.beat -= 4.0;
                        model.bar += 1;
                        // Process queued phrase at bar boundary
                        if let Some(queued) = model.queued_phrase.take() {
                            model.current_phrase = Some(queued);
                        }
                    }
                    return Cmd::delay(std::time::Duration::from_millis(50), Msg::Tick);
                }
            }
            Msg::SelectSetup(idx) => {
                model.current_setup = idx;
                model.current_phrase = None;
                model.queued_phrase = None;
            }
            Msg::SetSwitchMode(mode) => {
                model.switch_mode = mode;
            }
            Msg::SetQuantize(q) => {
                model.quantize = q;
            }
            Msg::SetParamMode(mode) => {
                model.param_mode = mode;
            }
            Msg::SelectPhrase(idx) => {
                model.current_phrase = Some(idx);
                model.queued_phrase = None;
            }
            Msg::QueuePhrase(idx) => {
                if model.current_phrase == Some(idx) {
                    model.queued_phrase = None;
                } else {
                    model.queued_phrase = Some(idx);
                }
            }
            Msg::ToggleTimingSection => {
                model.timing_section_open = !model.timing_section_open;
            }
            Msg::ToggleAudioSection => {
                model.audio_section_open = !model.audio_section_open;
            }
            Msg::ToggleMidiSection => {
                model.midi_section_open = !model.midi_section_open;
            }
            Msg::ToggleAudioReactivity => {
                model.audio_reactivity_on = !model.audio_reactivity_on;
            }

            // Lab Area
            Msg::SelectPatch(idx) => {
                model.current_patch = idx;
            }
            Msg::NewPatch => {
                let new_name = format!("patch-{}", model.patches.len() + 1);
                model.patches.push(Patch {
                    name: new_name,
                    ..Patch::default()
                });
                model.current_patch = model.patches.len() - 1;
            }
            Msg::DuplicatePatch => {
                if let Some(patch) = model.patches.get(model.current_patch).cloned() {
                    let mut new_patch = patch;
                    new_patch.name = format!("{}-copy", new_patch.name);
                    model.patches.push(new_patch);
                }
            }
            Msg::DeletePatch => {
                if model.patches.len() > 1 {
                    model.patches.remove(model.current_patch);
                    if model.current_patch >= model.patches.len() {
                        model.current_patch = model.patches.len() - 1;
                    }
                }
            }
            Msg::SetPatchName(name) => {
                if let Some(patch) = model.patches.get_mut(model.current_patch) {
                    patch.name = name;
                }
            }
            Msg::SetPatchAuthor(author) => {
                if let Some(patch) = model.patches.get_mut(model.current_patch) {
                    patch.author = author;
                }
            }
            Msg::SetPatchDesc(desc) => {
                if let Some(patch) = model.patches.get_mut(model.current_patch) {
                    patch.desc = desc;
                }
            }
            Msg::SetPatchTags(tags) => {
                if let Some(patch) = model.patches.get_mut(model.current_patch) {
                    patch.tags = tags;
                }
            }
            Msg::SetLabViewMode(mode) => {
                model.lab_view_mode = mode;
            }

            // MODs
            Msg::SetModBlendMode(mode) => {
                if let Some(m) = model.mods.first_mut() {
                    m.blend_mode = mode;
                }
            }
            Msg::SetModOpacityA(v) => {
                if let Some(m) = model.mods.first_mut() {
                    m.opacity_a = v;
                }
            }
            Msg::SetModOpacityB(v) => {
                if let Some(m) = model.mods.first_mut() {
                    m.opacity_b = v;
                }
            }
            Msg::SetModBitDepth(v) => {
                if let Some(m) = model.mods.first_mut() {
                    m.bit_depth = v;
                }
            }
            Msg::ToggleModEnabled => {
                if let Some(m) = model.mods.first_mut() {
                    m.enabled = !m.enabled;
                }
            }
            Msg::DeleteMod => {
                if !model.mods.is_empty() {
                    model.mods.remove(0);
                }
            }

            // Textures
            Msg::AddImageTexture => {
                model.textures.push(Texture {
                    name: format!("image_{}", model.textures.len() + 1),
                    tex_type: TextureType::Image,
                });
            }
            Msg::AddVideoTexture => {
                model.textures.push(Texture {
                    name: format!("video_{}", model.textures.len() + 1),
                    tex_type: TextureType::Video,
                });
            }
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        vj_theme().apply(ctx.ui.ctx());

        // Right Side Panel - Lab Area
        let mut lab_msgs: Vec<Msg> = Vec::new();
        egui::SidePanel::right("lab_panel")
            .min_width(400.0)
            .resizable(true)
            .show(ctx.ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_lab_area_ui(model, ui, &mut lab_msgs);
                });
            });
        for msg in lab_msgs {
            ctx.emit(msg);
        }

        // Central Panel - Live Area
        // Title
        ctx.ui.colored_label(egui::Color32::from_rgb(100, 150, 255),
            egui::RichText::new("Live").size(24.0));
        ctx.ui.separator();

        // Transport Row 1: BPM | Bar | Beat | Play | Rec
        render_transport_row1(model, ctx);

        // Transport Row 2: Source | Manual BPM | MIDI | Audio
        render_transport_row2(model, ctx);

        ctx.ui.add_space(8.0);

        // Setup Selector
        render_setup_selector(model, ctx);

        ctx.ui.add_space(8.0);

        // Performance State Card
        render_performance_state(model, ctx);

        ctx.ui.add_space(8.0);

        // Phrase Grid
        render_phrase_grid(model, ctx);

        ctx.ui.add_space(8.0);

        // Active Performance Status
        render_performance_status(model, ctx);

        ctx.ui.add_space(8.0);

        // Collapsible Sections
        render_timing_section(model, ctx);
        render_audio_section(model, ctx);
        render_midi_section(model, ctx);
    }
}

// ============================================================
// Transport
// ============================================================

fn render_transport_row1(model: &Model, ctx: &mut ViewCtx<Msg>) {
    ctx.horizontal(|ctx| {
        // BPM
        ctx.ui.label("BPM:");
        let mut bpm = model.bpm;
        if ctx.ui.add(
            egui::DragValue::new(&mut bpm)
                .range(30.0..=300.0)
                .speed(0.5)
                .fixed_decimals(1)
        ).changed() {
            ctx.emit(Msg::SetBpm(bpm));
        }

        ctx.ui.separator();

        // Bar & Beat
        let beat_int = model.beat.floor() as u32;
        ctx.ui.label(format!("Bar: {} | Beat: {}", model.bar, beat_int.max(1)));

        // Beat pulse indicator
        let beat_frac = model.beat.fract();
        let pulse_alpha = ((1.0 - beat_frac) * 150.0) as u8;
        let pulse_color = egui::Color32::from_rgba_unmultiplied(150, 150, 150, pulse_alpha);
        ctx.ui.colored_label(pulse_color, "□");

        ctx.ui.separator();

        // Play/Stop button
        if model.playing {
            semantics::stop(ButtonStyle::Both).on_click(ctx, Msg::TogglePlay);
        } else {
            semantics::play(ButtonStyle::Both).on_click(ctx, Msg::TogglePlay);
        }

        // Rec button
        Icon::record().size(14.0).color(if model.recording {
            egui::Color32::RED
        } else {
            egui::Color32::GRAY
        }).show(ctx.ui);
        if ctx.ui.button(if model.recording { "Rec" } else { "Rec" }).clicked() {
            ctx.emit(Msg::ToggleRecord);
        }
    });
}

fn render_transport_row2(model: &Model, ctx: &mut ViewCtx<Msg>) {
    ctx.horizontal(|ctx| {
        // Source
        ctx.ui.label(format!("Source: {}", model.timing_source.label()));

        ctx.ui.separator();

        // Manual BPM checkbox
        let mut manual = model.manual_bpm_enabled;
        if ctx.ui.checkbox(&mut manual, "Manual BPM").changed() {
            ctx.emit(Msg::ToggleManualBpm);
        }

        ctx.ui.separator();

        // MIDI status
        let midi_label = if model.midi_status { "MIDI: On" } else { "MIDI: Off" };
        let midi_color = if model.midi_status { egui::Color32::LIGHT_GREEN } else { egui::Color32::GRAY };
        ctx.ui.colored_label(midi_color, midi_label);

        // Audio status
        let audio_label = if model.audio_status { "Audio: On" } else { "Audio: Off" };
        let audio_color = if model.audio_status { egui::Color32::YELLOW } else { egui::Color32::GRAY };
        ctx.ui.colored_label(audio_color, audio_label);
    });
}

// ============================================================
// Setup Selector
// ============================================================

fn render_setup_selector(model: &Model, ctx: &mut ViewCtx<Msg>) {
    ctx.horizontal(|ctx| {
        ctx.ui.label("Setup:");

        // Setup dropdown/buttons
        for (i, setup) in model.setups.iter().enumerate() {
            let selected = model.current_setup == i;
            if ctx.ui.selectable_label(selected, &setup.name).clicked() {
                ctx.emit(Msg::SelectSetup(i));
            }
        }

        // Phrase count
        if let Some(setup) = model.setups.get(model.current_setup) {
            ctx.ui.colored_label(
                egui::Color32::GRAY,
                format!("({} phrases)", setup.phrases.len())
            );
        }
    });
}

// ============================================================
// Performance State
// ============================================================

fn render_performance_state(model: &Model, ctx: &mut ViewCtx<Msg>) {
    let mut pending_msg: Option<Msg> = None;

    egui::Frame::group(ctx.ui.style())
        .inner_margin(12.0)
        .show(ctx.ui, |ui| {
            // Playing
            ui.horizontal(|ui| {
                ui.label("Playing:");
                let playing_text = model.current_phrase
                    .and_then(|i| model.setups.get(model.current_setup)?.phrases.get(i))
                    .map(|p| p.name.as_str())
                    .unwrap_or("--");
                ui.label(playing_text);
            });

            // Queued
            ui.horizontal(|ui| {
                ui.label("Queued:");
                let queued_text = model.queued_phrase
                    .and_then(|i| model.setups.get(model.current_setup)?.phrases.get(i))
                    .map(|p| p.name.as_str())
                    .unwrap_or("(none)");
                ui.colored_label(egui::Color32::GRAY, queued_text);
            });

            ui.add_space(4.0);

            // Switch mode
            ui.horizontal(|ui| {
                ui.label("Switch:");
                if ui.selectable_label(model.switch_mode == SwitchMode::Auto, "Auto").clicked() {
                    pending_msg = Some(Msg::SetSwitchMode(SwitchMode::Auto));
                }
                if ui.selectable_label(model.switch_mode == SwitchMode::Manual, "Manual").clicked() {
                    pending_msg = Some(Msg::SetSwitchMode(SwitchMode::Manual));
                }
            });

            // Quantize
            ui.horizontal(|ui| {
                ui.label("Quantize:");
                for q in [Quantize::Immediate, Quantize::Beat, Quantize::Bar, Quantize::TwoBars, Quantize::FourBars] {
                    if ui.selectable_label(model.quantize == q, q.label()).clicked() {
                        pending_msg = Some(Msg::SetQuantize(q));
                    }
                }
            });

            // Params mode
            ui.horizontal(|ui| {
                ui.label("Params:");
                if ui.selectable_label(model.param_mode == ParamMode::Reset, "Reset").clicked() {
                    pending_msg = Some(Msg::SetParamMode(ParamMode::Reset));
                }
                if ui.selectable_label(model.param_mode == ParamMode::Inherit, "Inherit").clicked() {
                    pending_msg = Some(Msg::SetParamMode(ParamMode::Inherit));
                }
            });
        });

    if let Some(msg) = pending_msg {
        ctx.emit(msg);
    }
}

// ============================================================
// Phrase Grid
// ============================================================

fn render_phrase_grid(model: &Model, ctx: &mut ViewCtx<Msg>) {
    ctx.ui.label("Phrase Grid");

    let Some(setup) = model.setups.get(model.current_setup) else {
        ctx.ui.colored_label(egui::Color32::GRAY, "(No setup)");
        return;
    };

    if setup.phrases.is_empty() {
        ctx.ui.colored_label(egui::Color32::GRAY, "(No phrases)");
        return;
    }

    let tile_size = egui::Vec2::new(120.0, 70.0);
    let mut pending_msg: Option<Msg> = None;

    egui::Grid::new("phrase_grid")
        .spacing([8.0, 8.0])
        .show(ctx.ui, |ui| {
            for (i, phrase) in setup.phrases.iter().enumerate() {
                let is_current = model.current_phrase == Some(i);
                let is_queued = model.queued_phrase == Some(i);

                let bg_color = if is_current {
                    egui::Color32::from_rgb(60, 60, 90)
                } else if is_queued {
                    egui::Color32::from_rgb(90, 90, 60)
                } else {
                    egui::Color32::from_rgb(50, 50, 55)
                };

                let response = ui.add(
                    egui::Button::new("")
                        .fill(bg_color)
                        .min_size(tile_size)
                );

                // Draw phrase name
                let rect = response.rect;
                let painter = ui.painter();
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &phrase.name,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );

                // Status indicator
                if is_current || is_queued {
                    let icon_font = egui::FontId::new(12.0, egui::FontFamily::Name("icons".into()));
                    let (icon, color) = if is_current {
                        (icons::PLAY, egui::Color32::LIGHT_GREEN)
                    } else {
                        (icons::PAUSE, egui::Color32::YELLOW)
                    };
                    painter.text(
                        rect.right_top() + egui::Vec2::new(-12.0, 12.0),
                        egui::Align2::CENTER_CENTER,
                        icon,
                        icon_font,
                        color,
                    );
                }

                // Handle click
                if response.clicked() {
                    let shift_held = ui.input(|i| i.modifiers.shift);
                    if shift_held {
                        pending_msg = Some(Msg::SelectPhrase(i));
                    } else {
                        pending_msg = Some(Msg::QueuePhrase(i));
                    }
                }

                response.on_hover_text(format!(
                    "{}\n\nClick: Queue\nShift+Click: Switch Now",
                    phrase.name
                ));

                // 3 columns
                if (i + 1) % 3 == 0 {
                    ui.end_row();
                }
            }
        });

    if let Some(msg) = pending_msg {
        ctx.emit(msg);
    }
}

// ============================================================
// Performance Status
// ============================================================

fn render_performance_status(model: &Model, ctx: &mut ViewCtx<Msg>) {
    if model.current_phrase.is_none() {
        ctx.ui.colored_label(egui::Color32::GRAY, "(No active performance)");
    }
}

// ============================================================
// Collapsible Sections
// ============================================================

fn render_timing_section(model: &Model, ctx: &mut ViewCtx<Msg>) {
    let header = if model.timing_section_open { "▼ Timing & Beat Sync" } else { "▶ Timing & Beat Sync" };
    if ctx.ui.selectable_label(false, header).clicked() {
        ctx.emit(Msg::ToggleTimingSection);
    }

    if model.timing_section_open {
        ctx.ui.indent("timing_indent", |ui| {
            ui.label("Timing & Beat Sync");

            ui.horizontal(|ui| {
                ui.label("Source:");
                let _ = ui.selectable_label(true, "Audio Analysis");
                ui.label(format!("{:.1} BPM", model.bpm));
            });

            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::GRAY, "○");
                ui.label("MIDI Clock: Stopped");
            });

            let mut manual = model.manual_bpm_enabled;
            ui.checkbox(&mut manual, "Manual BPM Override");
        });
    }
}

fn render_audio_section(model: &Model, ctx: &mut ViewCtx<Msg>) {
    let header = if model.audio_section_open { "▼ Audio Reactivity" } else { "▶ Audio Reactivity" };
    if ctx.ui.selectable_label(false, header).clicked() {
        ctx.emit(Msg::ToggleAudioSection);
    }

    if model.audio_section_open {
        let mut toggle_audio = false;

        ctx.ui.indent("audio_indent", |ui| {
            ui.horizontal(|ui| {
                // Low
                ui.vertical(|ui| {
                    ui.label("Low");
                    ui.add(egui::ProgressBar::new(model.audio_low).desired_width(80.0));
                });

                // Mid
                ui.vertical(|ui| {
                    ui.label("Mid");
                    ui.add(egui::ProgressBar::new(model.audio_mid).desired_width(80.0));
                });

                // High
                ui.vertical(|ui| {
                    ui.label("High");
                    ui.add(egui::ProgressBar::new(model.audio_high).desired_width(80.0));
                });

                // ON/OFF button
                let label = if model.audio_reactivity_on { "ON" } else { "OFF" };
                if ui.button(label).clicked() {
                    toggle_audio = true;
                }
            });
        });

        if toggle_audio {
            ctx.emit(Msg::ToggleAudioReactivity);
        }
    }
}

fn render_midi_section(model: &Model, ctx: &mut ViewCtx<Msg>) {
    let header = if model.midi_section_open { "▼ MIDI Status" } else { "▶ MIDI Status" };
    if ctx.ui.selectable_label(false, header).clicked() {
        ctx.emit(Msg::ToggleMidiSection);
    }

    if model.midi_section_open {
        ctx.ui.indent("midi_indent", |ui| {
            ui.horizontal(|ui| {
                ui.label("MIDI");
                ui.colored_label(egui::Color32::GRAY, "Not connected");
            });
        });
    }
}

// ============================================================
// Lab Area (UI direct version for SidePanel)
// ============================================================

fn render_lab_area_ui(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    // Title
    Text::h2("Lab").color(Theme::current(ui.ctx()).info).show(ui);
    ui.separator();

    // Patch Tab Bar
    render_patch_tab_bar_ui(model, ui, msgs);

    ui.add_space(8.0);

    // Patch Edit Card
    render_patch_edit_card_ui(model, ui, msgs);

    ui.add_space(16.0);

    // MODs Section
    render_mods_section_ui(model, ui, msgs);

    ui.add_space(16.0);

    // Textures Section
    render_textures_section_ui(model, ui, msgs);
}

fn render_patch_tab_bar_ui(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        // New patch button
        if semantics::add(ButtonStyle::Icon).show(ui) {
            msgs.push(Msg::NewPatch);
        }

        // Copy button
        if semantics::copy(ButtonStyle::Icon).show(ui) {
            msgs.push(Msg::DuplicatePatch);
        }

        // Patch tabs
        for (i, patch) in model.patches.iter().enumerate() {
            let selected = model.current_patch == i;
            if selected {
                if Button::primary(&patch.name).show(ui) {
                    msgs.push(Msg::SelectPatch(i));
                }
            } else {
                if Button::outline(&patch.name).show(ui) {
                    msgs.push(Msg::SelectPatch(i));
                }
            }
        }
    });

    // Second row for view mode buttons
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Pack button
            if model.lab_view_mode == LabViewMode::Pack {
                Button::primary("Pack").show(ui);
            } else if Button::outline("Pack").show(ui) {
                msgs.push(Msg::SetLabViewMode(LabViewMode::Pack));
            }
            // Basic button
            if model.lab_view_mode == LabViewMode::Basic {
                Button::primary("Basic").show(ui);
            } else if Button::outline("Basic").show(ui) {
                msgs.push(Msg::SetLabViewMode(LabViewMode::Basic));
            }
            // YAML button
            if model.lab_view_mode == LabViewMode::Yaml {
                Button::primary("YAML").show(ui);
            } else if Button::outline("YAML").show(ui) {
                msgs.push(Msg::SetLabViewMode(LabViewMode::Yaml));
            }
            // Save button
            if semantics::save(ButtonStyle::Icon).show(ui) {
                // Save action (no-op for mock)
            }
        });
    });
}

fn render_patch_edit_card_ui(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    let Some(patch) = model.patches.get(model.current_patch) else {
        return;
    };

    Card::titled(&format!("Edit: {}", patch.name)).show(ui, |ui| {
        egui::Grid::new("patch_edit_grid")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                // Name
                Text::body("Name:").show(ui);
                let mut name = patch.name.clone();
                Input::new().show(ui, &mut name);
                if name != patch.name {
                    msgs.push(Msg::SetPatchName(name));
                }
                ui.end_row();

                // Author
                Text::body("Author:").show(ui);
                let mut author = patch.author.clone();
                Input::new().show(ui, &mut author);
                if author != patch.author {
                    msgs.push(Msg::SetPatchAuthor(author));
                }
                ui.end_row();

                // Desc
                Text::body("Desc:").show(ui);
                let mut desc = patch.desc.clone();
                Input::new().show(ui, &mut desc);
                if desc != patch.desc {
                    msgs.push(Msg::SetPatchDesc(desc));
                }
                ui.end_row();

                // Tags
                Text::body("Tags:").show(ui);
                let mut tags = patch.tags.clone();
                Input::new().placeholder("tag1, tag2, ...").show(ui, &mut tags);
                if tags != patch.tags {
                    msgs.push(Msg::SetPatchTags(tags));
                }
                ui.end_row();
            });

        ui.add_space(8.0);

        // Version & Created
        Text::caption(&format!("{}   Created: {}", patch.version, patch.created)).show(ui);

        ui.add_space(8.0);

        // Action buttons
        ui.horizontal(|ui| {
            if semantics::copy(ButtonStyle::Both).show(ui) {
                msgs.push(Msg::DuplicatePatch);
            }
            if semantics::delete(ButtonStyle::Both).show(ui) {
                msgs.push(Msg::DeletePatch);
            }
        });
    });
}

fn render_mods_section_ui(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        Text::h3("MODs").show(ui);

        // Dropdown (mock)
        egui::ComboBox::from_id_salt("mod_selector")
            .selected_text("[current]")
            .show_ui(ui, |ui| {
                let _ = ui.selectable_label(true, "[current]");
            });

        // Add buttons
        Button::outline("Builtin").show(ui);
        Button::outline("File").show(ui);
        Button::outline("Reg").show(ui);
    });

    ui.add_space(8.0);

    // Mod cards
    for (i, mod_effect) in model.mods.iter().enumerate() {
        Card::new().show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                // Speaker icon (enabled indicator)
                if mod_effect.enabled {
                    if ui.add(Button::ghost("").icon(icons::EYE)).clicked() {
                        msgs.push(Msg::ToggleModEnabled);
                    }
                } else {
                    if ui.add(Button::ghost("").icon(icons::EYE_SLASH)).clicked() {
                        msgs.push(Msg::ToggleModEnabled);
                    }
                }

                Text::body(&mod_effect.name).bold().show(ui);
                Badge::info(&format!("[{}]", mod_effect.effect_type)).show(ui);
                Text::caption(&mod_effect.node_id).show(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if semantics::delete(ButtonStyle::Icon).show(ui) {
                        msgs.push(Msg::DeleteMod);
                    }
                });
            });

            ui.add_space(8.0);

            // blend_mode
            Text::small("blend_mode").show(ui);
            ui.horizontal_wrapped(|ui| {
                for mode in BlendMode::all() {
                    let selected = mod_effect.blend_mode == *mode;
                    if selected {
                        if Button::primary(mode.label()).show(ui) {
                            msgs.push(Msg::SetModBlendMode(*mode));
                        }
                    } else {
                        if Button::ghost(mode.label()).show(ui) {
                            msgs.push(Msg::SetModBlendMode(*mode));
                        }
                    }
                }
            });

            ui.add_space(4.0);

            // Parameters
            render_mod_parameter_ui(ui, "opacity_a", mod_effect.opacity_a, msgs, i, |v| Msg::SetModOpacityA(v));
            render_mod_parameter_ui(ui, "opacity_b", mod_effect.opacity_b, msgs, i, |v| Msg::SetModOpacityB(v));
            render_mod_parameter_ui(ui, "bit_depth", mod_effect.bit_depth, msgs, i, |v| Msg::SetModBitDepth(v));
        });
    }
}

fn render_mod_parameter_ui<F>(
    ui: &mut egui::Ui,
    name: &str,
    value: f32,
    msgs: &mut Vec<Msg>,
    _mod_idx: usize,
    make_msg: F,
) where
    F: Fn(f32) -> Msg,
{
    ui.horizontal(|ui| {
        Text::small(name).show(ui);

        let mut val = value as f64;
        if Slider::new(0.0..=1.0).show_value(false).show(ui, &mut val) {
            msgs.push(make_msg(val as f32));
        }

        // Value display
        Text::body(&format!("{:.2}", value)).show(ui);

        // M button (MIDI learn)
        Button::ghost("M").show(ui);

        // Star (favorite)
        Button::ghost("☆").show(ui);
    });
}

fn render_textures_section_ui(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        Text::h3("Textures").show(ui);

        if semantics::add(ButtonStyle::Both).show(ui) {
            msgs.push(Msg::AddImageTexture);
        }
        if Button::outline("Video").show(ui) {
            msgs.push(Msg::AddVideoTexture);
        }
    });

    ui.add_space(4.0);

    if model.textures.is_empty() {
        Text::caption("(No textures loaded)").show(ui);
    } else {
        for tex in &model.textures {
            ui.horizontal(|ui| {
                match tex.tex_type {
                    TextureType::Image => Icon::new(icons::FILE).show(ui),
                    TextureType::Video => Icon::play().show(ui),
                };
                Text::body(&tex.name).show(ui);
            });
        }
    }
}
