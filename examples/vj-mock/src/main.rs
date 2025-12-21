//! VJ Software Mock - Live & Lab UI
//!
//! A mock VJ software UI showcasing egui-cha-ds VJ/DAW atoms:
//!
//! **Live Panel** (Performance):
//! - BpmDisplay, Transport, Timeline
//! - ClipGrid for phrase triggering
//! - Spectrum, LevelMeter, Waveform for audio visualization
//! - MidiMonitor for MIDI status
//! - CrossFader for A/B mixing
//!
//! **Lab Panel** (Editing):
//! - MediaBrowser for texture management
//! - EffectRack for MODs/effects
//! - ColorWheel for color adjustment
//! - LayerStack for layer management

use egui::Color32;
use egui_cha::prelude::*;
// icons is re-exported through prelude
use egui_cha_ds::prelude::*;

// ============================================================
// Theme
// ============================================================

fn vj_theme() -> Theme {
    Theme {
        variant: ThemeVariant::Dark,
        primary: Color32::from_rgb(0, 200, 220),
        primary_hover: Color32::from_rgb(0, 180, 200),
        primary_text: Color32::BLACK,
        secondary: Color32::from_rgb(80, 120, 130),
        secondary_hover: Color32::from_rgb(60, 100, 110),
        secondary_text: Color32::WHITE,
        bg_primary: Color32::from_rgb(8, 8, 12),
        bg_secondary: Color32::from_rgb(16, 18, 24),
        bg_tertiary: Color32::from_rgb(28, 32, 40),
        text_primary: Color32::from_rgb(230, 235, 240),
        text_secondary: Color32::from_rgb(160, 170, 180),
        text_muted: Color32::from_rgb(100, 110, 120),
        state_success: Color32::from_rgb(40, 200, 100),
        state_warning: Color32::from_rgb(230, 180, 40),
        state_danger: Color32::from_rgb(220, 60, 70),
        state_info: Color32::from_rgb(60, 160, 230),
        state_success_text: Color32::BLACK,
        state_warning_text: Color32::BLACK,
        state_danger_text: Color32::WHITE,
        state_info_text: Color32::BLACK,
        state_success_hover: Color32::from_rgb(30, 180, 80),
        state_warning_hover: Color32::from_rgb(210, 160, 30),
        state_danger_hover: Color32::from_rgb(200, 40, 50),
        state_info_hover: Color32::from_rgb(40, 140, 210),
        log_debug: Color32::from_rgb(100, 110, 120),
        log_info: Color32::from_rgb(60, 160, 230),
        log_warn: Color32::from_rgb(230, 180, 40),
        log_error: Color32::from_rgb(220, 60, 70),
        log_critical: Color32::from_rgb(200, 80, 180),
        border: Color32::from_rgb(50, 55, 65),
        border_focus: Color32::from_rgb(0, 200, 220),
        spacing_xs: 4.0,
        spacing_sm: 8.0,
        spacing_md: 16.0,
        spacing_lg: 24.0,
        spacing_xl: 32.0,
        radius_sm: 2.0,
        radius_md: 4.0,
        radius_lg: 6.0,
        border_width: 1.0,
        stroke_width: 1.0,
        font_size_xs: 10.0,
        font_size_sm: 12.0,
        font_size_md: 14.0,
        font_size_lg: 16.0,
        font_size_xl: 20.0,
        font_size_2xl: 24.0,
        font_size_3xl: 30.0,
        line_height: 1.4,
        overlay_dim: 0.8,
        surface_alpha: 1.0,
        shadow_blur: None,
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    egui_cha::run::<VjApp>(
        RunConfig::new("VJ Mock - Live & Lab")
            .with_size(1400.0, 900.0)
            .with_repaint_mode(RepaintMode::FixedFps(60)),
    )
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
    Tick,

    // Timeline
    TimelineSeek(f64),

    // Setup
    SelectSetup(usize),

    // Performance State
    SetSwitchMode(SwitchMode),
    SetQuantize(Quantize),

    // Phrase Grid (ClipGrid)
    TriggerClip(usize),

    // CrossFader
    CrossfaderChange(f32),

    // Collapsible sections
    ToggleAudioSection,
    ToggleMidiSection,

    // Lab Area
    SelectPatch(usize),
    NewPatch,
    DuplicatePatch,
    DeletePatch,
    SetPatchName(String),
    SetPatchAuthor(String),
    SetLabViewMode(LabViewMode),

    // Effects
    EffectEvent(RackEvent),

    // Textures (MediaBrowser)
    MediaEvent(MediaBrowserEvent),

    // Layers
    LayerEvent(LayerEvent),

    // Color Wheel
    ColorChange(Color32),

    // Output Router
    RouterEvent(RouterEvent),
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
enum LabViewMode {
    Yaml,
    Basic,
    Pack,
}

// ============================================================
// Model
// ============================================================

struct Model {
    // Transport
    playing: bool,
    recording: bool,
    bpm: f32,
    bar: u32,
    beat: f32,
    timeline_position: f64,

    // Setup
    setups: Vec<Setup>,
    current_setup: usize,

    // Performance State
    current_clip: Option<usize>,
    queued_clips: Vec<usize>,
    switch_mode: SwitchMode,
    quantize: Quantize,

    // CrossFader
    crossfader: f32,

    // Collapsible sections
    audio_section_open: bool,
    midi_section_open: bool,

    // Audio data (simulated)
    audio_samples: Vec<f32>,
    fft_bins: Vec<f32>,

    // MIDI data (simulated)
    midi_cc_values: Vec<CcValue>,
    midi_messages: Vec<MidiMessage>,

    // Lab Area
    patches: Vec<Patch>,
    current_patch: usize,
    lab_view_mode: LabViewMode,

    // Effects
    effects: Vec<Effect>,

    // Textures
    media_items: Vec<MediaItem>,
    selected_media: Option<String>,

    // Layers
    layers: Vec<Layer>,

    // Color
    tint_color: Color32,

    // Output Router
    route_sources: Vec<RouteSource>,
    route_outputs: Vec<RouteOutput>,
    route_connections: Vec<RouteConnection>,
}

struct Setup {
    name: String,
    clips: Vec<ClipCell>,
}

#[derive(Clone)]
struct Patch {
    name: String,
    author: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            playing: false,
            recording: false,
            bpm: 128.0,
            bar: 1,
            beat: 1.0,
            timeline_position: 0.0,

            setups: vec![
                Setup {
                    name: "Setup 1".into(),
                    clips: vec![
                        ClipCell::new("Intro").with_color(Color32::from_rgb(100, 150, 255)),
                        ClipCell::new("Build").with_color(Color32::from_rgb(150, 200, 100)),
                    ],
                },
                Setup {
                    name: "Setup 2".into(),
                    clips: vec![
                        ClipCell::new("Intro").with_color(Color32::from_rgb(120, 180, 255)),
                        ClipCell::new("Build").with_color(Color32::from_rgb(180, 255, 120)),
                        ClipCell::new("Drop")
                            .with_color(Color32::from_rgb(255, 100, 100))
                            .with_state(ClipState::Playing),
                        ClipCell::new("Break").with_color(Color32::from_rgb(255, 180, 100)),
                        ClipCell::new("Outro").with_color(Color32::from_rgb(200, 150, 255)),
                    ],
                },
            ],
            current_setup: 1,

            current_clip: Some(2),
            queued_clips: vec![],
            switch_mode: SwitchMode::Auto,
            quantize: Quantize::Immediate,

            crossfader: 0.0,

            audio_section_open: false,
            midi_section_open: false,

            audio_samples: vec![0.0; 128],
            fft_bins: vec![0.0; 64],

            midi_cc_values: vec![
                CcValue::new(1, 64).with_label("Mod"),
                CcValue::new(7, 100).with_label("Vol"),
                CcValue::new(10, 64).with_label("Pan"),
                CcValue::new(74, 80).with_label("Cutoff"),
            ],
            midi_messages: vec![],

            patches: vec![
                Patch {
                    name: "patch-1".into(),
                    author: "Unknown".into(),
                },
                Patch {
                    name: "patch-1-copy".into(),
                    author: "Unknown".into(),
                },
            ],
            current_patch: 0,
            lab_view_mode: LabViewMode::Basic,

            effects: vec![
                Effect::new("Blend", EffectCategory::Utility)
                    .enabled(true)
                    .with_param(EffectParam::new("opacity_a", 0.5))
                    .with_param(EffectParam::new("opacity_b", 0.5))
                    .with_param(EffectParam::new("bit_depth", 0.45)),
                Effect::new("Glitch", EffectCategory::Distortion)
                    .enabled(false)
                    .with_param(EffectParam::new("intensity", 0.3)),
                Effect::new("Blur", EffectCategory::Time)
                    .enabled(true)
                    .with_param(EffectParam::new("amount", 0.2)),
            ],

            media_items: vec![
                MediaItem::new("1", "video_1").with_type(MediaType::Video),
                MediaItem::new("2", "image_2").with_type(MediaType::Image),
                MediaItem::new("3", "video_3").with_type(MediaType::Video),
                MediaItem::new("4", "image_4").with_type(MediaType::Image),
                MediaItem::new("5", "video_5").with_type(MediaType::Video),
                MediaItem::new("6", "image_6").with_type(MediaType::Image),
            ],
            selected_media: Some("1".into()),

            layers: vec![
                Layer::new("Main Output")
                    .with_opacity(1.0)
                    .with_visible(true),
                Layer::new("Overlay").with_opacity(0.8).with_visible(true),
                Layer::new("Background")
                    .with_opacity(0.5)
                    .with_visible(false),
            ],

            tint_color: Color32::from_rgb(0, 200, 220),

            route_sources: vec![
                RouteSource::new("main", "Main Mix").with_type(SourceType::Main),
                RouteSource::new("preview", "Preview").with_type(SourceType::Preview),
                RouteSource::new("layer1", "Layer 1").with_type(SourceType::Layer),
            ],
            route_outputs: vec![
                RouteOutput::new("proj", "Projector")
                    .with_type(OutputType::Display)
                    .with_resolution(1920, 1080),
                RouteOutput::new("led", "LED Wall")
                    .with_type(OutputType::Display)
                    .with_resolution(1280, 720),
                RouteOutput::new("ndi", "NDI Out").with_type(OutputType::NDI),
                RouteOutput::new("rec", "Record").with_type(OutputType::Record),
            ],
            route_connections: vec![
                RouteConnection::new("main", "proj"),
                RouteConnection::new("main", "ndi"),
                RouteConnection::new("preview", "led"),
            ],
        }
    }
}

// ============================================================
// App Implementation
// ============================================================

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
            Msg::Tick => {
                if model.playing {
                    let beat_per_tick = model.bpm / 60.0 * 0.05;
                    model.beat += beat_per_tick;
                    model.timeline_position = (model.timeline_position + 0.001) % 1.0;

                    if model.beat > 4.0 {
                        model.beat -= 4.0;
                        model.bar += 1;
                        // Process queued clips at bar boundary
                        if let Some(queued) = model.queued_clips.first().copied() {
                            model.current_clip = Some(queued);
                            model.queued_clips.remove(0);
                            // Update clip states
                            if let Some(setup) = model.setups.get_mut(model.current_setup) {
                                for (i, clip) in setup.clips.iter_mut().enumerate() {
                                    if Some(i) == model.current_clip {
                                        *clip = clip.clone().with_state(ClipState::Playing);
                                    } else if model.queued_clips.contains(&i) {
                                        *clip = clip.clone().with_state(ClipState::Queued);
                                    } else {
                                        *clip = clip.clone().with_state(ClipState::Idle);
                                    }
                                }
                            }
                        }
                    }

                    // Simulate audio data
                    let time = model.bar as f32 + model.beat / 4.0;
                    for (i, sample) in model.audio_samples.iter_mut().enumerate() {
                        let t = i as f32 / 128.0 * std::f32::consts::PI * 4.0 + time * 2.0;
                        *sample = t.sin() * 0.6 + (t * 2.0).sin() * 0.3;
                    }
                    for (i, bin) in model.fft_bins.iter_mut().enumerate() {
                        let freq = i as f32 / 64.0;
                        let base = (1.0 - freq).powf(1.5);
                        let pulse = (time * 2.0 + i as f32 * 0.1).sin() * 0.5 + 0.5;
                        *bin = (base * pulse * 0.8).clamp(0.0, 1.0);
                    }

                    return Cmd::delay(std::time::Duration::from_millis(50), Msg::Tick);
                }
            }
            Msg::TimelineSeek(pos) => {
                model.timeline_position = pos;
            }
            Msg::SelectSetup(idx) => {
                model.current_setup = idx;
                model.current_clip = None;
                model.queued_clips.clear();
            }
            Msg::SetSwitchMode(mode) => {
                model.switch_mode = mode;
            }
            Msg::SetQuantize(q) => {
                model.quantize = q;
            }
            Msg::TriggerClip(idx) => {
                if model.quantize == Quantize::Immediate {
                    model.current_clip = Some(idx);
                } else {
                    if !model.queued_clips.contains(&idx) {
                        model.queued_clips.push(idx);
                    }
                }
                // Update clip states
                if let Some(setup) = model.setups.get_mut(model.current_setup) {
                    for (i, clip) in setup.clips.iter_mut().enumerate() {
                        if Some(i) == model.current_clip {
                            *clip = clip.clone().with_state(ClipState::Playing);
                        } else if model.queued_clips.contains(&i) {
                            *clip = clip.clone().with_state(ClipState::Queued);
                        } else {
                            *clip = clip.clone().with_state(ClipState::Idle);
                        }
                    }
                }
            }
            Msg::CrossfaderChange(v) => {
                model.crossfader = v;
            }
            Msg::ToggleAudioSection => {
                model.audio_section_open = !model.audio_section_open;
            }
            Msg::ToggleMidiSection => {
                model.midi_section_open = !model.midi_section_open;
            }

            // Lab
            Msg::SelectPatch(idx) => {
                model.current_patch = idx;
            }
            Msg::NewPatch => {
                let name = format!("patch-{}", model.patches.len() + 1);
                model.patches.push(Patch {
                    name,
                    author: "Unknown".into(),
                });
                model.current_patch = model.patches.len() - 1;
            }
            Msg::DuplicatePatch => {
                if let Some(p) = model.patches.get(model.current_patch).cloned() {
                    model.patches.push(Patch {
                        name: format!("{}-copy", p.name),
                        author: p.author,
                    });
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
                if let Some(p) = model.patches.get_mut(model.current_patch) {
                    p.name = name;
                }
            }
            Msg::SetPatchAuthor(author) => {
                if let Some(p) = model.patches.get_mut(model.current_patch) {
                    p.author = author;
                }
            }
            Msg::SetLabViewMode(mode) => {
                model.lab_view_mode = mode;
            }
            Msg::EffectEvent(event) => match event {
                RackEvent::Toggle(idx) => {
                    if let Some(e) = model.effects.get_mut(idx) {
                        e.enabled = !e.enabled;
                    }
                }
                RackEvent::Remove(idx) => {
                    if idx < model.effects.len() {
                        model.effects.remove(idx);
                    }
                }
                _ => {}
            },
            Msg::MediaEvent(event) => match event {
                MediaBrowserEvent::Select(id) => {
                    model.selected_media = Some(id);
                }
                _ => {}
            },
            Msg::LayerEvent(event) => match event {
                LayerEvent::ToggleVisible(idx) => {
                    if let Some(l) = model.layers.get_mut(idx) {
                        l.visible = !l.visible;
                    }
                }
                LayerEvent::SetOpacity(idx, opacity) => {
                    if let Some(l) = model.layers.get_mut(idx) {
                        l.opacity = opacity;
                    }
                }
                _ => {}
            },
            Msg::ColorChange(color) => {
                model.tint_color = color;
            }
            Msg::RouterEvent(event) => match event {
                RouterEvent::Connect {
                    source_id,
                    output_id,
                } => {
                    model
                        .route_connections
                        .push(RouteConnection::new(source_id, output_id));
                }
                RouterEvent::Disconnect {
                    source_id,
                    output_id,
                } => {
                    model
                        .route_connections
                        .retain(|c| !(c.source_id == source_id && c.output_id == output_id));
                }
                RouterEvent::ToggleOutput(id) => {
                    if let Some(o) = model.route_outputs.iter_mut().find(|o| o.id == id) {
                        o.enabled = !o.enabled;
                    }
                }
                _ => {}
            },
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        vj_theme().apply(ctx.ui.ctx());

        // Right Side Panel - Lab Area
        let mut lab_msgs: Vec<Msg> = Vec::new();
        egui::SidePanel::right("lab_panel")
            .min_width(420.0)
            .resizable(true)
            .show(ctx.ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_lab_area(model, ui, &mut lab_msgs);
                });
            });
        for msg in lab_msgs {
            ctx.emit(msg);
        }

        // Main Central Panel - Live Area
        let mut live_msgs: Vec<Msg> = Vec::new();
        egui::CentralPanel::default().show(ctx.ui.ctx(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                render_live_area_ui(model, ui, &mut live_msgs);
            });
        });
        for msg in live_msgs {
            ctx.emit(msg);
        }
    }
}

// ============================================================
// Live Area
// ============================================================

fn render_live_area_ui(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    // Title
    Text::h1("Live")
        .color(Color32::from_rgb(100, 180, 255))
        .show(ui);
    ui.separator();

    // Transport Row
    ui.horizontal(|ui| {
        // BPM Display
        ui.label("BPM:");
        BpmDisplay::new().show(ui, model.bpm as f64);

        ui.add_space(16.0);

        // Bar & Beat
        let beat_int = model.beat.floor() as u32;
        ui.label(format!("Bar: {} | Beat: {}", model.bar, beat_int.max(1)));

        ui.add_space(16.0);

        // Transport buttons
        if model.playing {
            if semantics::stop(ButtonStyle::Both).show(ui) {
                msgs.push(Msg::TogglePlay);
            }
        } else {
            if semantics::play(ButtonStyle::Both).show(ui) {
                msgs.push(Msg::TogglePlay);
            }
        }

        // Record button
        let rec_color = if model.recording {
            Color32::RED
        } else {
            Color32::GRAY
        };
        Icon::record().size(16.0).color(rec_color).show(ui);
        if Button::ghost("Rec").show(ui) {
            msgs.push(Msg::ToggleRecord);
        }

        ui.add_space(16.0);

        // Status badges
        Badge::info("MIDI: Off").show(ui);
        Badge::new("Audio: Off").show(ui);
    });

    ui.add_space(8.0);

    // Timeline
    let markers = vec![
        TimelineMarker::new(0.0, "Start"),
        TimelineMarker::new(0.25, "A").with_color(Color32::from_rgb(100, 200, 100)),
        TimelineMarker::new(0.5, "B").with_color(Color32::from_rgb(200, 100, 200)),
        TimelineMarker::new(0.75, "C").with_color(Color32::from_rgb(200, 200, 100)),
    ];

    if let Some(event) = Timeline::new(120.0)
        .position(model.timeline_position)
        .markers(&markers)
        .height(32.0)
        .show(ui)
    {
        match event {
            TimelineEvent::Seek(p) => msgs.push(Msg::TimelineSeek(p)),
            _ => {}
        }
    }

    ui.add_space(12.0);

    // Setup Selector
    ui.horizontal(|ui| {
        ui.label("Setup:");
        for (i, setup) in model.setups.iter().enumerate() {
            let selected = model.current_setup == i;
            if selected {
                if Button::primary(&setup.name).show(ui) {
                    msgs.push(Msg::SelectSetup(i));
                }
            } else {
                if Button::outline(&setup.name).show(ui) {
                    msgs.push(Msg::SelectSetup(i));
                }
            }
        }
        if let Some(setup) = model.setups.get(model.current_setup) {
            Text::caption(&format!("({} clips)", setup.clips.len())).show(ui);
        }
    });

    ui.add_space(8.0);

    // Performance State Card
    Card::new().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Playing:");
            let playing_name = model
                .current_clip
                .and_then(|i| model.setups.get(model.current_setup)?.clips.get(i))
                .map(|c| c.name.as_str())
                .unwrap_or("--");
            Text::body(playing_name).bold().show(ui);

            ui.add_space(16.0);

            ui.label("Queued:");
            if model.queued_clips.is_empty() {
                Text::caption("(none)").show(ui);
            } else {
                for &idx in &model.queued_clips {
                    if let Some(clip) = model
                        .setups
                        .get(model.current_setup)
                        .and_then(|s| s.clips.get(idx))
                    {
                        Badge::warning(&clip.name).show(ui);
                    }
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Switch:");
            if ui
                .selectable_label(model.switch_mode == SwitchMode::Auto, "Auto")
                .clicked()
            {
                // emit via return
            }
            if ui
                .selectable_label(model.switch_mode == SwitchMode::Manual, "Manual")
                .clicked()
            {
                // emit via return
            }

            ui.add_space(16.0);

            ui.label("Quantize:");
            for q in [
                Quantize::Immediate,
                Quantize::Beat,
                Quantize::Bar,
                Quantize::TwoBars,
                Quantize::FourBars,
            ] {
                if ui
                    .selectable_label(model.quantize == q, q.label())
                    .clicked()
                {
                    // emit via return
                }
            }
        });
    });

    ui.add_space(12.0);

    // Phrase Grid using ClipGrid
    ui.label("Phrase Grid");
    if let Some(setup) = model.setups.get(model.current_setup) {
        if let Some(idx) = ClipGrid::new(&setup.clips, 4)
            .current(model.current_clip)
            .queued(&model.queued_clips)
            .cell_size(100.0, 60.0)
            .show(ui)
        {
            msgs.push(Msg::TriggerClip(idx));
        }
    }

    ui.add_space(12.0);

    // CrossFader
    ui.label("A/B Mix");
    if let Some(value) = CrossFader::new()
        .value(model.crossfader)
        .labels("Deck A", "Deck B")
        .curve(CrossfaderCurve::EqualPower)
        .size(400.0, 36.0)
        .show(ui)
    {
        msgs.push(Msg::CrossfaderChange(value));
    }

    ui.add_space(16.0);

    // Audio Reactivity Section (Collapsible)
    let audio_header = if model.audio_section_open {
        "▼ Audio Reactivity"
    } else {
        "▶ Audio Reactivity"
    };
    if ui.selectable_label(false, audio_header).clicked() {
        msgs.push(Msg::ToggleAudioSection);
    }

    if model.audio_section_open {
        ui.indent("audio_section", |ui| {
            ui.label("Waveform");
            Waveform::new(&model.audio_samples)
                .height(50.0)
                .filled()
                .show(ui);

            ui.add_space(8.0);

            ui.label("Spectrum");
            Spectrum::new(&model.fft_bins).height(60.0).show(ui);

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Levels:");
                let time = (model.bar as f32 + model.beat / 4.0) * 0.5;
                let level_l = -60.0 + (time.sin() * 30.0 + 30.0);
                let level_r = -60.0 + ((time + 0.5).sin() * 30.0 + 30.0);
                LevelMeter::new().size(20.0, 80.0).show(ui, level_l);
                LevelMeter::new().size(20.0, 80.0).show(ui, level_r);
            });
        });
    }

    ui.add_space(8.0);

    // MIDI Section (Collapsible)
    let midi_header = if model.midi_section_open {
        "▼ MIDI Status"
    } else {
        "▶ MIDI Status"
    };
    if ui.selectable_label(false, midi_header).clicked() {
        msgs.push(Msg::ToggleMidiSection);
    }

    if model.midi_section_open {
        ui.indent("midi_section", |ui| {
            MidiMonitor::new()
                .device_name("MIDI Controller")
                .cc_values(&model.midi_cc_values)
                .messages(&model.midi_messages)
                .mode(MonitorMode::CcGrid)
                .size(350.0, 100.0)
                .show(ui);
        });
    }
}

// ============================================================
// Lab Area
// ============================================================

fn render_lab_area(model: &Model, ui: &mut egui::Ui, msgs: &mut Vec<Msg>) {
    // Title
    Text::h1("Lab")
        .color(Color32::from_rgb(100, 200, 255))
        .show(ui);
    ui.separator();

    // Patch Tab Bar
    ui.horizontal(|ui| {
        if semantics::add(ButtonStyle::Icon).show(ui) {
            msgs.push(Msg::NewPatch);
        }
        if semantics::copy(ButtonStyle::Icon).show(ui) {
            msgs.push(Msg::DuplicatePatch);
        }

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

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for (mode, label) in [
                (LabViewMode::Pack, "Pack"),
                (LabViewMode::Basic, "Basic"),
                (LabViewMode::Yaml, "YAML"),
            ] {
                if model.lab_view_mode == mode {
                    Button::primary(label).show(ui);
                } else if Button::outline(label).show(ui) {
                    msgs.push(Msg::SetLabViewMode(mode));
                }
            }
            if semantics::save(ButtonStyle::Icon).show(ui) {}
        });
    });

    ui.add_space(8.0);

    // Patch Edit Card
    if let Some(patch) = model.patches.get(model.current_patch) {
        Card::titled(&format!("Edit: {}", patch.name)).show(ui, |ui| {
            egui::Grid::new("patch_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    Text::body("Name:").show(ui);
                    let mut name = patch.name.clone();
                    Input::new().show(ui, &mut name);
                    if name != patch.name {
                        msgs.push(Msg::SetPatchName(name));
                    }
                    ui.end_row();

                    Text::body("Author:").show(ui);
                    let mut author = patch.author.clone();
                    Input::new().show(ui, &mut author);
                    if author != patch.author {
                        msgs.push(Msg::SetPatchAuthor(author));
                    }
                    ui.end_row();
                });

            ui.add_space(8.0);
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

    ui.add_space(16.0);

    // MODs Section - using EffectRack
    Text::h3("MODs (Effects)").show(ui);
    ui.add_space(4.0);

    if let Some(event) = EffectRack::new(&model.effects)
        .orientation(RackOrientation::Vertical)
        .effect_size(380.0, 70.0)
        .show(ui)
    {
        msgs.push(Msg::EffectEvent(event));
    }

    ui.add_space(16.0);

    // Textures Section - using MediaBrowser
    ui.horizontal(|ui| {
        Text::h3("Textures").show(ui);
        if semantics::add(ButtonStyle::Icon).show(ui) {
            // Add texture action
        }
    });
    ui.add_space(4.0);

    if let Some(event) = MediaBrowser::new(&model.media_items)
        .selected(model.selected_media.as_deref())
        .view_mode(BrowserViewMode::List)
        .size(380.0, 150.0)
        .show(ui)
    {
        msgs.push(Msg::MediaEvent(event));
    }

    ui.add_space(16.0);

    // Layers Section - using LayerStack
    Text::h3("Layers").show(ui);
    ui.add_space(4.0);

    if let Some(event) = LayerStack::new(&model.layers).selected(Some(0)).show(ui) {
        msgs.push(Msg::LayerEvent(event));
    }

    ui.add_space(16.0);

    // Tint Color - using ColorWheel
    Text::h3("Tint Color").show(ui);
    ui.add_space(4.0);

    let mut tint = model.tint_color;
    if ColorWheel::new().size(120.0).show(ui, &mut tint).changed() {
        msgs.push(Msg::ColorChange(tint));
    }

    ui.add_space(40.0);

    // Output Router
    Text::h3("Output Router").show(ui);
    ui.add_space(8.0);

    if let Some(event) = OutputRouter::new(
        &model.route_sources,
        &model.route_outputs,
        &model.route_connections,
    )
    .size(380.0, 200.0)
    .show(ui)
    {
        msgs.push(Msg::RouterEvent(event));
    }

    // Bottom padding for scroll area
    ui.add_space(16.0);
}
