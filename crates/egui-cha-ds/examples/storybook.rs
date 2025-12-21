//! DS Storybook - Component showcase
//!
//! Visual catalog of all DS components and framework features

use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;
#[cfg(feature = "dock")]
use egui_cha_ds::{dock_layout, DockArea, DockEvent, DockTree};
#[cfg(feature = "snarl")]
use egui_cha_ds::{
    NodeGraph, NodeGraphArea, NodeGraphEvent,
    NodeId, InPin, OutPin, PinInfo, Snarl, SnarlViewer,
    NodeLayout, NodeLayoutArea, LayoutPane,
};
use egui_cha_ds::{ConfirmResult, ToastContainer, ToastId};
use std::cell::RefCell;
use std::time::Duration;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    egui_cha::run::<StorybookApp>(
        RunConfig::new("DS Storybook")
            .with_size(1200.0, 800.0),
    )
}

struct StorybookApp;

#[derive(Default)]
struct Model {
    // Navigation
    active_category: usize,
    active_component: usize,

    // Component states
    button_clicks: u32,
    checkbox_value: bool,
    toggle_value: bool,
    slider_value: f64,
    knob_value: f64,
    knob_value2: f64,
    knob_value3: f64,
    fader_value: f64,
    fader_value2: f64,
    fader_value3: f64,
    fader_value4: f64,
    xypad_value: (f64, f64),
    xypad_value2: (f64, f64),
    arc_value: f64,
    arc_value2: f64,
    arc_value3: f64,
    group_value: f64,
    group_value2: f64,
    input_value: String,
    select_value: usize,

    // Validation demo
    email_value: String,
    email_validation: ValidationState,
    password_value: String,
    password_validation: ValidationState,

    // Modal state
    show_modal: bool,
    show_confirm: bool,
    confirm_result: Option<bool>,

    // Theme
    theme: Theme,
    theme_index: usize,  // 0: Light, 1: Dark, 2: Pastel, 3: Pastel Dark

    // Theme scale settings
    spacing_scale: f32,
    radius_scale: f32,
    font_scale: f32,
    stroke_scale: f32,
    shadow_enabled: bool,
    shadow_blur: f32,
    overlay_dim: f32,

    // Tabs demo
    tabs_index: usize,
    menu_index: usize,

    // Table demo
    table_data: Vec<(String, i32, bool)>,

    // === Framework Demo States ===

    // Cmd demo
    delay_count: u32,
    delay_pending: bool,
    timeout_status: Option<&'static str>,
    retry_status: Option<String>,
    retry_attempt: u32,

    // Sub::interval demo
    interval_enabled: bool,
    interval_count: u32,

    // Debouncer demo
    debounce_input: String,
    debounce_search_count: u32,
    debouncer: Debouncer,

    // Throttler demo
    throttle_click_count: u32,
    throttle_actual_count: u32,
    throttler: Throttler,

    // Columns demo
    col_clicks: [u32; 4],

    // Conditionals demo
    cond_show: bool,
    cond_enabled: bool,
    cond_visible: bool,

    // Toast demo
    toasts: ToastContainer,

    // Form demo
    form_submitted: bool,

    // Drag & Drop demo
    dnd_items: Vec<String>,
    dnd_dropped: Vec<String>,

    // Keyboard shortcuts demo
    shortcut_counter: i32,
    shortcut_last_action: Option<&'static str>,

    // Dynamic bindings demo
    bindings: ActionBindings<DemoAction>,
    bindings_counter: i32,
    bindings_last_action: Option<&'static str>,
    bindings_rebind_mode: bool,

    // Context menu demo
    context_menu_last_action: Option<&'static str>,

    // ErrorConsole demo
    error_console: ErrorConsoleState,

    // Dock demo (RefCell for interior mutability in view)
    dock: RefCell<DockTree<DemoPane>>,

    // === VJ/DAW Demo States ===

    // MIDI Keyboard demo
    keyboard_notes: Vec<ActiveNote>,

    // MIDI Monitor demo
    midi_messages: Vec<MidiMessage>,
    midi_cc_values: Vec<CcValue>,

    // Piano Roll demo
    piano_notes: Vec<MidiNote>,
    piano_position: f32,
    piano_selected: Option<usize>,

    // Mixer demo
    channel_volumes: [f32; 4],
    channel_pans: [f32; 4],
    channel_mutes: [bool; 4],
    channel_solos: [bool; 4],

    // Crossfader demo
    crossfader_value: f32,

    // Timeline demo
    timeline_position: f64,

    // Color wheel demo
    wheel_color: Hsva,

    // NodeGraph demo (RefCell for interior mutability in view)
    #[cfg(feature = "snarl")]
    node_graph: RefCell<NodeGraph<DemoNode>>,
    #[cfg(feature = "snarl")]
    node_graph_last_event: Option<String>,

    // NodeLayout demo
    node_layout: RefCell<NodeLayout>,
    node_layout_lock_level: egui_cha_ds::LockLevel,
    node_layout_show_menu_bar: bool,
}

/// Demo node type for NodeGraph showcase
#[cfg(feature = "snarl")]
#[derive(Clone, Debug)]
enum DemoNode {
    Source { name: String },
    Effect { intensity: f32 },
    Output,
}

/// Demo action for dynamic bindings showcase
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum DemoAction {
    Increment,
    Decrement,
    Reset,
    Save,
}

/// Demo pane for dock showcase
#[derive(Clone, Debug, PartialEq, Default)]
enum DemoPane {
    #[default]
    Browser,
    Editor,
    Console,
    Inspector,
}

/// Demo viewer for NodeGraph showcase
#[cfg(feature = "snarl")]
#[derive(Default)]
struct DemoNodeViewer;

#[cfg(feature = "snarl")]
#[allow(refining_impl_trait)]
impl SnarlViewer<DemoNode> for DemoNodeViewer {
    fn title(&mut self, node: &DemoNode) -> String {
        match node {
            DemoNode::Source { name } => format!("Source: {}", name),
            DemoNode::Effect { .. } => "Effect".into(),
            DemoNode::Output => "Output".into(),
        }
    }

    fn inputs(&mut self, node: &DemoNode) -> usize {
        match node {
            DemoNode::Source { .. } => 0,
            DemoNode::Effect { .. } => 1,
            DemoNode::Output => 1,
        }
    }

    fn outputs(&mut self, node: &DemoNode) -> usize {
        match node {
            DemoNode::Source { .. } => 1,
            DemoNode::Effect { .. } => 1,
            DemoNode::Output => 0,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        _ui: &mut egui::Ui,
        snarl: &mut Snarl<DemoNode>,
    ) -> PinInfo {
        // Color and shape based on node type
        match snarl.get_node(pin.id.node) {
            Some(DemoNode::Effect { .. }) => {
                // Audio signal input - cyan circle
                PinInfo::circle().with_fill(egui::Color32::from_rgb(100, 200, 200))
            }
            Some(DemoNode::Output) => {
                // Final output - green triangle
                PinInfo::triangle().with_fill(egui::Color32::from_rgb(100, 200, 100))
            }
            _ => PinInfo::circle().with_fill(egui::Color32::GRAY),
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        _ui: &mut egui::Ui,
        snarl: &mut Snarl<DemoNode>,
    ) -> PinInfo {
        // Color and shape based on node type
        match snarl.get_node(pin.id.node) {
            Some(DemoNode::Source { .. }) => {
                // Audio source output - orange square
                PinInfo::square().with_fill(egui::Color32::from_rgb(255, 180, 100))
            }
            Some(DemoNode::Effect { .. }) => {
                // Processed audio output - cyan circle
                PinInfo::circle().with_fill(egui::Color32::from_rgb(100, 200, 200))
            }
            _ => PinInfo::circle().with_fill(egui::Color32::GRAY),
        }
    }

    fn has_body(&mut self, node: &DemoNode) -> bool {
        matches!(node, DemoNode::Effect { .. })
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<DemoNode>,
    ) {
        if let Some(DemoNode::Effect { intensity }) = snarl.get_node_mut(node) {
            ui.add(egui::Slider::new(intensity, 0.0..=1.0).text("Intensity"));
        }
    }
}

#[derive(Clone, Debug)]
enum Msg {
    // Navigation
    SetCategory(usize),
    SetComponent(usize),

    // Button
    ButtonClicked,

    // Checkbox/Toggle
    ToggleCheckbox,
    ToggleToggle,

    // Slider
    SliderChanged(f64),

    // Knob
    KnobChanged(f64),
    Knob2Changed(f64),
    Knob3Changed(f64),

    // Fader
    FaderChanged(f64),
    Fader2Changed(f64),
    Fader3Changed(f64),
    Fader4Changed(f64),

    // XYPad
    XYPadChanged((f64, f64)),
    XYPad2Changed((f64, f64)),

    // ArcSlider
    ArcChanged(f64),
    Arc2Changed(f64),
    Arc3Changed(f64),

    // ButtonGroup
    GroupChanged(f64),
    Group2Changed(f64),

    // Input
    InputChanged(String),

    // Select
    SelectChanged(usize),

    // Validation
    EmailChanged(String),
    PasswordChanged(String),

    // Modal
    OpenModal,
    CloseModal,
    OpenConfirm,
    ConfirmResult(bool),

    // Theme
    ToggleTheme,
    SetSpacingScale(f32),
    SetRadiusScale(f32),
    SetFontScale(f32),
    SetStrokeScale(f32),
    ToggleShadow,
    SetShadowBlur(f32),
    SetOverlayDim(f32),
    ResetThemeScales,

    // Tabs
    TabChanged(usize),

    // Menu
    MenuChanged(usize),

    // === Framework Demo Messages ===

    // Cmd::delay demo
    StartDelay,
    DelayComplete,

    // Cmd::with_timeout demo
    StartTimeout,
    TimeoutSuccess,
    TimeoutFailed,

    // Cmd::retry demo
    StartRetry,
    RetrySuccess,
    RetryFailed(String, u32),
    RetryAttempt,

    // Sub::interval demo
    ToggleInterval,
    IntervalTick,

    // Debouncer demo
    DebounceInput(String),
    DebounceSearch,

    // Throttler demo
    ThrottleClick,
    ThrottleActual,

    // Columns demo
    ColClick(usize),

    // Conditionals demo
    ToggleCondShow,
    ToggleCondEnabled,
    ToggleCondVisible,

    // Toast demo
    ShowToastInfo,
    ShowToastSuccess,
    ShowToastWarning,
    ShowToastError,
    DismissToast(ToastId),

    // Form demo
    FormSubmit,

    // Drag & Drop demo
    DndDropped(String),

    // Keyboard shortcuts demo
    ShortcutIncrement,
    ShortcutDecrement,
    ShortcutReset,
    ShortcutSave,
    ShortcutUndo,

    // Dynamic bindings demo
    BindingsAction(DemoAction),
    BindingsRebind(DemoAction, DynamicShortcut),
    BindingsReset(DemoAction),
    BindingsResetAll,
    BindingsToggleRebindMode,

    // Context menu demo
    ContextMenuEdit,
    ContextMenuCopy,
    ContextMenuDelete,

    // ErrorConsole demo
    ErrorConsolePush(ErrorLevel),
    ErrorConsoleMsg(ErrorConsoleMsg),

    // Dock
    DockEvent(DockEvent<DemoPane>),

    // NodeGraph
    #[cfg(feature = "snarl")]
    NodeGraphEvent(NodeGraphEvent<DemoNode>),

    // NodeLayout
    ToggleNodeLayoutLock,
    SetNodeLayoutLock(egui_cha_ds::LockLevel),
    ToggleNodeLayoutMenuBar,

    // === VJ/DAW Demo Messages ===

    // MIDI Keyboard
    KeyboardNoteOn(u8, u8),
    KeyboardNoteOff(u8),

    // Piano Roll
    PianoNoteAdd(u8, f32, f32),
    PianoNoteMove(usize, u8, f32),
    PianoNoteDelete(usize),
    PianoNoteSelect(usize),
    PianoSeek(f32),

    // Channel Strip
    ChannelVolume(usize, f32),
    ChannelPan(usize, f32),
    ChannelMute(usize),
    ChannelSolo(usize),

    // Crossfader
    CrossfaderChange(f32),

    // Timeline
    TimelineSeek(f64),

    // Color Wheel
    ColorWheelChange(Hsva),
}

const CATEGORIES: &[&str] = &[
    "Core",       // 0: Basic UI atoms
    "Audio",      // 1: Audio visualization
    "MIDI",       // 2: MIDI components
    "Mixer",      // 3: Mixing & effects
    "Visual",     // 4: Visual editing
    "Semantics",  // 5
    "Molecules",  // 6
    "Framework",  // 7
    "Theme",      // 8
];

// Core atoms - Basic UI components (always available)
const CORE_ATOMS: &[&str] = &[
    "Button",
    "Badge",
    "Icon",
    "Input",
    "Checkbox",
    "Toggle",
    "Slider",
    "Knob",
    "Fader",
    "XYPad",
    "ArcSlider",
    "ButtonGroup",
    "Link",
    "Code",
    "Text",
    "Tooltip",
    "Context Menu",
    "ListItem",
    "Select",
];

// Audio atoms - Audio visualization & control
const AUDIO_ATOMS: &[&str] = &[
    "Waveform",
    "Spectrum",
    "LevelMeter",
    "Oscilloscope",
    "BpmDisplay",
    "Transport",
    "BeatSync",
    "StepSeq",
    "SamplePad",
];

// MIDI atoms - MIDI input & editing
const MIDI_ATOMS: &[&str] = &[
    "MidiKeyboard",
    "MidiMonitor",
    "MidiMapper",
    "PianoRoll",
];

// Mixer atoms - Audio mixing & effects
const MIXER_ATOMS: &[&str] = &[
    "ChannelStrip",
    "CrossFader",
    "EffectRack",
    "EnvelopeEditor",
    "AutomationLane",
];

// Visual atoms - Video/graphics editing
const VISUAL_ATOMS: &[&str] = &[
    "ClipGrid",
    "Timeline",
    "Preview",
    "LayerStack",
    "ColorWheel",
    "GradientEditor",
    "MaskEditor",
    "TransformGizmo",
    "MediaBrowser",
    "OutputRouter",
];

// Plot atoms (feature-gated)
const PLOT_ATOMS: &[&str] = &["Plot"];

const SEMANTICS: &[&str] = &[
    "Overview",
    "File Operations",
    "Actions",
    "Media",
    "Navigation",
    "ButtonStyle",
    "SeverityLog",
];

const MOLECULES: &[&str] = &[
    "Card",
    "Tabs",
    "Menu",
    "Modal",
    "Table",
    "Navbar",
    "ErrorConsole",
    "Toast",
    "Form",
    "Columns",
    "Conditionals",
    "Dock",
    "NodeGraph",
    "NodeLayout",
];

const FRAMEWORK: &[&str] = &[
    "Cmd::delay",
    "Cmd::timeout",
    "Cmd::retry",
    "Sub::interval",
    "Debouncer",
    "Throttler",
    "Drag & Drop",
    "Shortcuts",
    "Dynamic Bindings",
    "ScrollArea",
    "RepaintMode",
];

const THEME_ITEMS: &[&str] = &[
    "Scale Controls",
    "Shadow & Overlay",
    "Preview",
];

/// Rebuild theme from model settings
fn rebuild_theme(model: &mut Model) {
    let base = match model.theme_index {
        0 => Theme::light(),
        1 => Theme::dark(),
        2 => Theme::pastel(),
        _ => Theme::pastel_dark(),
    };

    let mut theme = base
        .with_spacing_scale(model.spacing_scale)
        .with_radius_scale(model.radius_scale)
        .with_font_scale(model.font_scale)
        .with_stroke_scale(model.stroke_scale);

    // Apply shadow setting
    if model.shadow_enabled {
        theme = theme.with_shadow_blur(model.shadow_blur);
    }

    // Apply overlay_dim
    theme.overlay_dim = model.overlay_dim;

    model.theme = theme;
}

impl App for StorybookApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        // Set up default bindings for the demo
        let bindings = ActionBindings::new()
            .with_default(DemoAction::Increment, DynamicShortcut::new(Modifiers::NONE, Key::ArrowUp))
            .with_default(DemoAction::Decrement, DynamicShortcut::new(Modifiers::NONE, Key::ArrowDown))
            .with_default(DemoAction::Reset, DynamicShortcut::new(Modifiers::NONE, Key::Escape))
            .with_default(DemoAction::Save, shortcuts::SAVE);

        (
            Model {
                slider_value: 50.0,
                input_value: "Hello".to_string(),
                email_value: String::new(),
                email_validation: ValidationState::None,
                password_value: String::new(),
                password_validation: ValidationState::None,
                table_data: vec![
                    ("Alice".to_string(), 25, true),
                    ("Bob".to_string(), 30, false),
                    ("Carol".to_string(), 28, true),
                ],
                dnd_items: vec![
                    "Item A".to_string(),
                    "Item B".to_string(),
                    "Item C".to_string(),
                ],
                dnd_dropped: Vec::new(),
                theme: Theme::light(),
                spacing_scale: 1.0,
                radius_scale: 1.0,
                font_scale: 1.0,
                stroke_scale: 1.0,
                shadow_enabled: false,
                shadow_blur: 4.0,
                overlay_dim: 0.5,
                bindings,
                dock: RefCell::new(dock_layout::three_column(
                    DemoPane::Browser,
                    DemoPane::Editor,
                    DemoPane::Inspector,
                    0.2,
                    0.2,
                )),
                #[cfg(feature = "snarl")]
                node_graph: RefCell::new({
                    let mut graph = NodeGraph::new();
                    let source = graph.insert(egui::pos2(50.0, 100.0), DemoNode::Source { name: "Audio".into() });
                    let effect = graph.insert(egui::pos2(250.0, 100.0), DemoNode::Effect { intensity: 0.5 });
                    let output = graph.insert(egui::pos2(450.0, 100.0), DemoNode::Output);
                    // Connect source -> effect -> output
                    graph.connect(
                        egui_cha_ds::OutPinId { node: source, output: 0 },
                        egui_cha_ds::InPinId { node: effect, input: 0 },
                    );
                    graph.connect(
                        egui_cha_ds::OutPinId { node: effect, output: 0 },
                        egui_cha_ds::InPinId { node: output, input: 0 },
                    );
                    graph
                }),
                node_layout: RefCell::new({
                    let mut layout = NodeLayout::new();
                    layout.add_pane(
                        LayoutPane::new("preview", "Preview")
                            .with_size(280.0, 180.0)
                            .with_icon(egui_cha_ds::icons::MONITOR_PLAY)
                            .closable(true),
                        egui::pos2(20.0, 20.0),
                    );
                    layout.add_pane(
                        LayoutPane::new("effects", "Effects")
                            .with_size(200.0, 150.0)
                            .with_icon(egui_cha_ds::icons::SLIDERS_HORIZONTAL)
                            .closable(true),
                        egui::pos2(320.0, 20.0),
                    );
                    layout.add_pane(
                        LayoutPane::new("layers", "Layers")
                            .with_size(250.0, 120.0)
                            .with_icon(egui_cha_ds::icons::STACK)
                            .closable(true),
                        egui::pos2(20.0, 220.0),
                    );
                    layout
                }),
                node_layout_lock_level: egui_cha_ds::LockLevel::None,
                node_layout_show_menu_bar: true,
                ..Default::default()
            },
            Cmd::none(),
        )
    }

    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::SetCategory(idx) => {
                model.active_category = idx;
                model.active_component = 0;
            }
            Msg::SetComponent(idx) => {
                model.active_component = idx;
            }
            Msg::ButtonClicked => {
                model.button_clicks += 1;
            }
            Msg::ToggleCheckbox => {
                model.checkbox_value = !model.checkbox_value;
            }
            Msg::ToggleToggle => {
                model.toggle_value = !model.toggle_value;
            }
            Msg::SliderChanged(v) => {
                model.slider_value = v;
            }
            Msg::KnobChanged(v) => {
                model.knob_value = v;
            }
            Msg::Knob2Changed(v) => {
                model.knob_value2 = v;
            }
            Msg::Knob3Changed(v) => {
                model.knob_value3 = v;
            }
            Msg::FaderChanged(v) => {
                model.fader_value = v;
            }
            Msg::Fader2Changed(v) => {
                model.fader_value2 = v;
            }
            Msg::Fader3Changed(v) => {
                model.fader_value3 = v;
            }
            Msg::Fader4Changed(v) => {
                model.fader_value4 = v;
            }
            Msg::XYPadChanged(v) => {
                model.xypad_value = v;
            }
            Msg::XYPad2Changed(v) => {
                model.xypad_value2 = v;
            }
            Msg::ArcChanged(v) => {
                model.arc_value = v;
            }
            Msg::Arc2Changed(v) => {
                model.arc_value2 = v;
            }
            Msg::Arc3Changed(v) => {
                model.arc_value3 = v;
            }
            Msg::GroupChanged(v) => {
                model.group_value = v;
            }
            Msg::Group2Changed(v) => {
                model.group_value2 = v;
            }
            Msg::InputChanged(v) => {
                model.input_value = v;
            }
            Msg::EmailChanged(v) => {
                // Simple email validation
                model.email_validation = if v.is_empty() {
                    ValidationState::None
                } else if v.contains('@') && v.contains('.') {
                    ValidationState::Valid
                } else {
                    ValidationState::invalid("Please enter a valid email address")
                };
                model.email_value = v;
            }
            Msg::PasswordChanged(v) => {
                // Simple password validation
                model.password_validation = if v.is_empty() {
                    ValidationState::None
                } else if v.len() >= 8 {
                    ValidationState::Valid
                } else {
                    ValidationState::invalid("Password must be at least 8 characters")
                };
                model.password_value = v;
            }
            Msg::SelectChanged(idx) => {
                model.select_value = idx;
            }
            Msg::OpenModal => {
                model.show_modal = true;
            }
            Msg::CloseModal => {
                model.show_modal = false;
            }
            Msg::OpenConfirm => {
                model.show_confirm = true;
            }
            Msg::ConfirmResult(result) => {
                model.show_confirm = false;
                model.confirm_result = Some(result);
            }
            Msg::ToggleTheme => {
                model.theme_index = (model.theme_index + 1) % 4;
                rebuild_theme(model);
            }
            Msg::SetSpacingScale(v) => {
                model.spacing_scale = v;
                rebuild_theme(model);
            }
            Msg::SetRadiusScale(v) => {
                model.radius_scale = v;
                rebuild_theme(model);
            }
            Msg::SetFontScale(v) => {
                model.font_scale = v;
                rebuild_theme(model);
            }
            Msg::SetStrokeScale(v) => {
                model.stroke_scale = v;
                rebuild_theme(model);
            }
            Msg::ToggleShadow => {
                model.shadow_enabled = !model.shadow_enabled;
                rebuild_theme(model);
            }
            Msg::SetShadowBlur(v) => {
                model.shadow_blur = v;
                rebuild_theme(model);
            }
            Msg::SetOverlayDim(v) => {
                model.overlay_dim = v;
                rebuild_theme(model);
            }
            Msg::ResetThemeScales => {
                model.spacing_scale = 1.0;
                model.radius_scale = 1.0;
                model.font_scale = 1.0;
                model.stroke_scale = 1.0;
                model.shadow_enabled = false;
                model.shadow_blur = 4.0;
                model.overlay_dim = 0.5;
                rebuild_theme(model);
            }
            Msg::TabChanged(idx) => {
                model.tabs_index = idx;
            }
            Msg::MenuChanged(idx) => {
                model.menu_index = idx;
            }

            // === Framework Demo ===

            // Cmd::delay
            Msg::StartDelay => {
                model.delay_pending = true;
                return Cmd::delay(Duration::from_secs(2), Msg::DelayComplete);
            }
            Msg::DelayComplete => {
                model.delay_pending = false;
                model.delay_count += 1;
            }

            // Cmd::with_timeout
            Msg::StartTimeout => {
                model.timeout_status = Some("Loading...");
                // Fast task (completes in ~10ms) with 1s timeout - will succeed
                return Cmd::with_timeout(
                    Duration::from_secs(1),
                    async { 42 },
                    |_| Msg::TimeoutSuccess,
                    Msg::TimeoutFailed,
                );
            }
            Msg::TimeoutSuccess => {
                model.timeout_status = Some("Success!");
            }
            Msg::TimeoutFailed => {
                model.timeout_status = Some("Timeout!");
            }

            // Cmd::retry
            Msg::StartRetry => {
                model.retry_status = Some("Retrying...".to_string());
                model.retry_attempt = 0;
                // Simulate failing task that succeeds on 3rd attempt
                let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
                let attempt_clone = attempt.clone();
                return Cmd::retry(
                    3,
                    Duration::from_millis(500),
                    move || {
                        let attempt = attempt_clone.clone();
                        async move {
                            let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            if n < 2 {
                                Err(format!("Attempt {} failed", n + 1))
                            } else {
                                Ok("Success on attempt 3!")
                            }
                        }
                    },
                    |msg: &'static str| Msg::RetrySuccess,
                    |err, attempts| Msg::RetryFailed(err, attempts),
                );
            }
            Msg::RetrySuccess => {
                model.retry_status = Some("Succeeded after retries!".to_string());
            }
            Msg::RetryFailed(err, attempts) => {
                model.retry_status = Some(format!("Failed after {} attempts: {}", attempts, err));
            }
            Msg::RetryAttempt => {
                model.retry_attempt += 1;
            }

            // Sub::interval
            Msg::ToggleInterval => {
                model.interval_enabled = !model.interval_enabled;
                if !model.interval_enabled {
                    model.interval_count = 0;
                }
            }
            Msg::IntervalTick => {
                model.interval_count += 1;
            }

            // Debouncer
            Msg::DebounceInput(text) => {
                model.debounce_input = text;
                return model.debouncer.trigger(Duration::from_millis(500), Msg::DebounceSearch);
            }
            Msg::DebounceSearch => {
                if model.debouncer.should_fire() {
                    model.debounce_search_count += 1;
                }
            }

            // Throttler
            Msg::ThrottleClick => {
                model.throttle_click_count += 1;
                return model.throttler.run(Duration::from_millis(500), || {
                    Cmd::msg(Msg::ThrottleActual)
                });
            }
            Msg::ThrottleActual => {
                model.throttle_actual_count += 1;
            }

            // Columns demo
            Msg::ColClick(i) => {
                model.col_clicks[i] += 1;
            }

            // Conditionals demo
            Msg::ToggleCondShow => {
                model.cond_show = !model.cond_show;
            }
            Msg::ToggleCondEnabled => {
                model.cond_enabled = !model.cond_enabled;
            }
            Msg::ToggleCondVisible => {
                model.cond_visible = !model.cond_visible;
            }

            // Toast demo
            Msg::ShowToastInfo => {
                return model.toasts.info("This is an info message", Duration::from_secs(3), Msg::DismissToast);
            }
            Msg::ShowToastSuccess => {
                return model.toasts.success("Operation completed successfully!", Duration::from_secs(3), Msg::DismissToast);
            }
            Msg::ShowToastWarning => {
                return model.toasts.warning("Please check your input", Duration::from_secs(3), Msg::DismissToast);
            }
            Msg::ShowToastError => {
                return model.toasts.error("Something went wrong", Duration::from_secs(5), Msg::DismissToast);
            }
            Msg::DismissToast(id) => {
                model.toasts.dismiss(id);
            }

            // Form demo
            Msg::FormSubmit => {
                model.form_submitted = true;
                return model.toasts.success("Form submitted!", Duration::from_secs(3), Msg::DismissToast);
            }

            // Drag & Drop
            Msg::DndDropped(item) => {
                // Move item from source to dropped list
                if let Some(pos) = model.dnd_items.iter().position(|i| *i == item) {
                    model.dnd_items.remove(pos);
                }
                model.dnd_dropped.push(item);
            }

            // Keyboard shortcuts
            Msg::ShortcutIncrement => {
                model.shortcut_counter += 1;
                model.shortcut_last_action = Some("Increment (+)");
            }
            Msg::ShortcutDecrement => {
                model.shortcut_counter -= 1;
                model.shortcut_last_action = Some("Decrement (-)");
            }
            Msg::ShortcutReset => {
                model.shortcut_counter = 0;
                model.shortcut_last_action = Some("Reset (Escape)");
            }
            Msg::ShortcutSave => {
                model.shortcut_last_action = Some("Save (Cmd+S)");
            }
            Msg::ShortcutUndo => {
                model.shortcut_last_action = Some("Undo (Cmd+Z)");
            }

            // Dynamic bindings
            Msg::BindingsAction(action) => {
                match action {
                    DemoAction::Increment => {
                        model.bindings_counter += 1;
                        model.bindings_last_action = Some("Increment");
                    }
                    DemoAction::Decrement => {
                        model.bindings_counter -= 1;
                        model.bindings_last_action = Some("Decrement");
                    }
                    DemoAction::Reset => {
                        model.bindings_counter = 0;
                        model.bindings_last_action = Some("Reset");
                    }
                    DemoAction::Save => {
                        model.bindings_last_action = Some("Save");
                    }
                }
            }
            Msg::BindingsRebind(action, shortcut) => {
                model.bindings.rebind(&action, shortcut);
            }
            Msg::BindingsReset(action) => {
                model.bindings.reset(&action);
            }
            Msg::BindingsResetAll => {
                model.bindings.reset_all();
            }
            Msg::BindingsToggleRebindMode => {
                model.bindings_rebind_mode = !model.bindings_rebind_mode;
            }

            // Context menu demo
            Msg::ContextMenuEdit => {
                model.context_menu_last_action = Some("Edit");
            }
            Msg::ContextMenuCopy => {
                model.context_menu_last_action = Some("Copy");
            }
            Msg::ContextMenuDelete => {
                model.context_menu_last_action = Some("Delete");
            }

            // ErrorConsole demo
            Msg::ErrorConsolePush(level) => {
                let msg = match level {
                    ErrorLevel::Debug => "Debug: Internal state dump",
                    ErrorLevel::Info => "Info: Operation completed",
                    ErrorLevel::Warning => "Warning: Rate limit approaching",
                    ErrorLevel::Error => "Error: Failed to save data",
                    ErrorLevel::Critical => "Critical: Database connection lost",
                };
                model.error_console.push_with_level(msg, level);
            }
            Msg::ErrorConsoleMsg(msg) => match msg {
                ErrorConsoleMsg::Dismiss(i) => model.error_console.dismiss(i),
                ErrorConsoleMsg::DismissAll => model.error_console.clear(),
            },

            Msg::DockEvent(event) => {
                // Handle dock events (tab closed, add clicked, etc.)
                match event {
                    DockEvent::TabClosed(_tab) => {
                        // Tab was closed - could add it back or handle
                    }
                    DockEvent::AddClicked { surface: _, node: _ } => {
                        // Add button clicked - could add a new tab
                        model.dock.borrow_mut().push(DemoPane::Console);
                    }
                    DockEvent::FocusChanged => {
                        // Focus changed
                    }
                }
            }

            #[cfg(feature = "snarl")]
            Msg::NodeGraphEvent(event) => {
                model.node_graph_last_event = Some(format!("{:?}", event));
            }

            Msg::ToggleNodeLayoutLock => {
                model.node_layout_lock_level = model.node_layout_lock_level.cycle();
            }

            Msg::SetNodeLayoutLock(level) => {
                model.node_layout_lock_level = level;
            }

            Msg::ToggleNodeLayoutMenuBar => {
                model.node_layout_show_menu_bar = !model.node_layout_show_menu_bar;
            }

            // === VJ/DAW Demo Messages ===

            Msg::KeyboardNoteOn(note, velocity) => {
                model.keyboard_notes.push(ActiveNote::new(note, velocity));
            }
            Msg::KeyboardNoteOff(note) => {
                model.keyboard_notes.retain(|n| n.note != note);
            }
            Msg::PianoNoteAdd(note, start, duration) => {
                model.piano_notes.push(MidiNote::new(note, start, duration));
            }
            Msg::PianoNoteMove(idx, note, start) => {
                if let Some(n) = model.piano_notes.get_mut(idx) {
                    n.note = note;
                    n.start = start;
                }
            }
            Msg::PianoNoteDelete(idx) => {
                if idx < model.piano_notes.len() {
                    model.piano_notes.remove(idx);
                }
            }
            Msg::PianoNoteSelect(idx) => {
                model.piano_selected = Some(idx);
            }
            Msg::PianoSeek(pos) => {
                model.piano_position = pos;
            }
            Msg::ChannelVolume(idx, vol) => {
                if idx < 4 {
                    model.channel_volumes[idx] = vol;
                }
            }
            Msg::ChannelPan(idx, pan) => {
                if idx < 4 {
                    model.channel_pans[idx] = pan;
                }
            }
            Msg::ChannelMute(idx) => {
                if idx < 4 {
                    model.channel_mutes[idx] = !model.channel_mutes[idx];
                }
            }
            Msg::ChannelSolo(idx) => {
                if idx < 4 {
                    model.channel_solos[idx] = !model.channel_solos[idx];
                }
            }
            Msg::CrossfaderChange(val) => {
                model.crossfader_value = val;
            }
            Msg::TimelineSeek(pos) => {
                model.timeline_position = pos;
            }
            Msg::ColorWheelChange(color) => {
                model.wheel_color = color;
            }
        }
        Cmd::none()
    }

    fn subscriptions(model: &Model) -> Sub<Msg> {
        if model.interval_enabled {
            Sub::interval("demo_interval", Duration::from_secs(1), Msg::IntervalTick)
        } else {
            Sub::none()
        }
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        model.theme.apply(ctx.ui.ctx());

        // Use sidebar_layout for clean two-panel design
        ctx.sidebar_layout(
            "storybook_sidebar",
            220.0,
            // Sidebar: Navigation
            |ctx| {
                ctx.horizontal(|ctx| {
                    ctx.ui.heading("DS Storybook");
                });

                let theme_label = match model.theme_index {
                    0 => "Light > Dark",
                    1 => "Dark > Pastel",
                    2 => "Pastel > Pastel Dark",
                    _ => "Pastel Dark > Light",
                };
                Button::ghost(theme_label).on_click(ctx, Msg::ToggleTheme);

                ctx.ui.separator();
                ctx.ui.strong("Categories");
                ctx.ui.add_space(4.0);

                // Category selection
                Menu::new(CATEGORIES).compact().show_with(ctx, model.active_category, Msg::SetCategory);

                ctx.ui.add_space(8.0);
                ctx.ui.separator();
                ctx.ui.strong("Components");
                ctx.ui.add_space(4.0);

                // Component list based on category
                let components = match model.active_category {
                    0 => CORE_ATOMS,
                    1 => AUDIO_ATOMS,
                    2 => MIDI_ATOMS,
                    3 => MIXER_ATOMS,
                    4 => VISUAL_ATOMS,
                    5 => SEMANTICS,
                    6 => MOLECULES,
                    7 => FRAMEWORK,
                    _ => THEME_ITEMS,
                };
                Menu::new(components).compact().show_with(ctx, model.active_component, Msg::SetComponent);
            },
            // Main: Component preview
            |ctx| {
                ctx.ui.heading("Preview");
                ctx.ui.separator();

                Card::new().show_ctx(ctx, |ctx| {
                    match model.active_category {
                        0 => render_core_atom(model, ctx),
                        1 => render_audio_atom(model, ctx),
                        2 => render_midi_atom(model, ctx),
                        3 => render_mixer_atom(model, ctx),
                        4 => render_visual_atom(model, ctx),
                        5 => render_semantics(model, ctx),
                        6 => render_molecule(model, ctx),
                        7 => render_framework(model, ctx),
                        _ => render_theme(model, ctx),
                    }
                });

                // Modals (inside main panel)
                if model.show_modal {
                    let close = Modal::titled("Demo Modal")
                        .show(ctx.ui, true, |ui| {
                            ui.label("This is a modal dialog.");
                            ui.label("You can put any content here.");
                            ui.label("Click the X button or backdrop to close.");
                        });
                    if close {
                        ctx.emit(Msg::CloseModal);
                    }
                }

                if model.show_confirm {
                    let result = ConfirmDialog::new("Confirm Action", "Are you sure you want to proceed?")
                        .show(ctx.ui, true);
                    match result {
                        ConfirmResult::Confirmed => ctx.emit(Msg::ConfirmResult(true)),
                        ConfirmResult::Cancelled => ctx.emit(Msg::ConfirmResult(false)),
                        ConfirmResult::None => {}
                    }
                }
            },
        );

        // Show toasts (overlay)
        model.toasts.show(ctx, Msg::DismissToast);
    }
}

fn render_core_atom(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match CORE_ATOMS[model.active_component] {
        "Button" => {
            ctx.ui.heading("Button");
            ctx.ui.label("Various button styles");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Button::primary("Primary").on_click(ctx, Msg::ButtonClicked);
                Button::secondary("Secondary").on_click(ctx, Msg::ButtonClicked);
                Button::outline("Outline").on_click(ctx, Msg::ButtonClicked);
                Button::ghost("Ghost").on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Clicked: {} times", model.button_clicks));
        }

        "Badge" => {
            ctx.ui.heading("Badge");
            ctx.ui.label("Status indicators");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Badge::new("Default").show(ctx.ui);
                Badge::success("Success").show(ctx.ui);
                Badge::warning("Warning").show(ctx.ui);
                Badge::error("Error").show(ctx.ui);
                Badge::info("Info").show(ctx.ui);
            });
        }

        "Icon" => {
            ctx.ui.heading("Icon");
            ctx.ui.label("Phosphor icons");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Icon::house().size(24.0).show(ctx.ui);
                Icon::gear().size(24.0).show(ctx.ui);
                Icon::user().size(24.0).show(ctx.ui);
                Icon::check().size(24.0).show(ctx.ui);
                Icon::warning().size(24.0).show(ctx.ui);
                Icon::info().size(24.0).show(ctx.ui);
                Icon::plus().size(24.0).show(ctx.ui);
                Icon::minus().size(24.0).show(ctx.ui);
                Icon::arrow_left().size(24.0).show(ctx.ui);
                Icon::arrow_right().size(24.0).show(ctx.ui);
            });

            ctx.ui.add_space(8.0);
            ctx.ui.label("Different sizes:");
            ctx.horizontal(|ctx| {
                Icon::house().size(16.0).show(ctx.ui);
                Icon::house().size(24.0).show(ctx.ui);
                Icon::house().size(32.0).show(ctx.ui);
                Icon::house().size(48.0).show(ctx.ui);
            });
        }

        "Input" => {
            ctx.ui.heading("Input");
            ctx.ui.label("Text input field");
            ctx.ui.add_space(8.0);

            Input::new()
                .placeholder("Type something...")
                .show_with(ctx, &model.input_value, Msg::InputChanged);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Value: {}", model.input_value));

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.heading("Validated Input");
            ctx.ui.label("Input with validation state (try typing)");
            ctx.ui.add_space(8.0);

            ValidatedInput::new("Email")
                .placeholder("user@example.com")
                .show_with(
                    &model.email_value,
                    &model.email_validation,
                    ctx,
                    Msg::EmailChanged,
                );

            ctx.ui.add_space(12.0);

            ValidatedInput::new("Password")
                .placeholder("Enter password")
                .password()
                .show_with(
                    &model.password_value,
                    &model.password_validation,
                    ctx,
                    Msg::PasswordChanged,
                );
        }

        "Checkbox" => {
            ctx.ui.heading("Checkbox");
            ctx.ui.label("Boolean toggle with label");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                let mut value = model.checkbox_value;
                if ctx.ui.checkbox(&mut value, "Enable feature").changed() {
                    ctx.emit(Msg::ToggleCheckbox);
                }
            });

            ctx.ui.label(format!("Value: {}", model.checkbox_value));
        }

        "Toggle" => {
            ctx.ui.heading("Toggle");
            ctx.ui.label("Switch-style boolean toggle");
            ctx.ui.add_space(8.0);

            Toggle::with_label("Dark mode")
                .show_with(ctx, model.toggle_value, |_| Msg::ToggleToggle);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Value: {}", model.toggle_value));
        }

        "Slider" => {
            ctx.ui.heading("Slider");
            ctx.ui.label("Numeric range input");
            ctx.ui.add_space(8.0);

            Slider::new(0.0_f64..=100.0_f64)
                .show_with(ctx, model.slider_value, Msg::SliderChanged);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Value: {:.1}", model.slider_value));
        }

        "Knob" => {
            ctx.ui.heading("Knob");
            ctx.ui.label("Rotary knob for EDM/VJ applications");
            ctx.ui.add_space(16.0);

            // Row of knobs with different sizes
            ctx.horizontal(|ctx| {
                ctx.vertical(|ctx| {
                    Knob::new(0.0..=100.0)
                        .compact()
                        .label("Compact")
                        .show_with(ctx, model.knob_value, Msg::KnobChanged);
                });

                ctx.ui.add_space(16.0);

                ctx.vertical(|ctx| {
                    Knob::new(0.0..=1.0)
                        .label("Medium")
                        .show_with(ctx, model.knob_value2, Msg::Knob2Changed);
                });

                ctx.ui.add_space(16.0);

                ctx.vertical(|ctx| {
                    Knob::new(0.0..=100.0)
                        .size(KnobSize::Large)
                        .label("Large")
                        .show_with(ctx, model.knob_value3, Msg::Knob3Changed);
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Usage example
            ctx.ui.label("Usage:");
            Code::new(r#"Knob::new(0.0..=100.0)
    .label("Volume")
    .show_with(ctx, model.volume, Msg::SetVolume);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Drag up/down to adjust value");
            ctx.ui.label("• Double-click to reset to center");
            ctx.ui.label("• Theme-aware styling");
        }

        "Fader" => {
            ctx.ui.heading("Fader");
            ctx.ui.label("Vertical fader for mixer-style controls");
            ctx.ui.add_space(16.0);

            // Row of faders (mixer style)
            ctx.horizontal(|ctx| {
                ctx.vertical(|ctx| {
                    Fader::new(0.0..=1.0)
                        .compact()
                        .label("CH1")
                        .show_with(ctx, model.fader_value, Msg::FaderChanged);
                });

                ctx.ui.add_space(8.0);

                ctx.vertical(|ctx| {
                    Fader::new(0.0..=1.0)
                        .compact()
                        .label("CH2")
                        .show_with(ctx, model.fader_value2, Msg::Fader2Changed);
                });

                ctx.ui.add_space(8.0);

                ctx.vertical(|ctx| {
                    Fader::new(0.0..=1.0)
                        .label("CH3")
                        .show_with(ctx, model.fader_value3, Msg::Fader3Changed);
                });

                ctx.ui.add_space(16.0);

                ctx.vertical(|ctx| {
                    Fader::new(-60.0..=6.0)
                        .size(FaderSize::Large)
                        .label("Master")
                        .db_scale(true)
                        .show_with(ctx, model.fader_value4, Msg::Fader4Changed);
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"Fader::new(-60.0..=6.0)
    .label("Master")
    .db_scale(true)
    .show_with(ctx, model.master, Msg::SetMaster);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Drag to adjust, click to set");
            ctx.ui.label("• Double-click to reset (0dB for dB scale)");
            ctx.ui.label("• dB scale with -∞ display");
        }

        "XYPad" => {
            ctx.ui.heading("XYPad");
            ctx.ui.label("2D touch pad for X/Y parameter control (Kaoss Pad style)");
            ctx.ui.add_space(16.0);

            ctx.horizontal(|ctx| {
                // Basic XY pad
                ctx.vertical(|ctx| {
                    ctx.ui.label("Basic:");
                    XYPad::new()
                        .size(150.0, 150.0)
                        .show_with(ctx, model.xypad_value, Msg::XYPadChanged);
                });

                ctx.ui.add_space(24.0);

                // With grid and labels
                ctx.vertical(|ctx| {
                    ctx.ui.label("With grid:");
                    XYPad::new()
                        .size(150.0, 150.0)
                        .grid(true)
                        .label_x("Cutoff")
                        .label_y("Q")
                        .show_with(ctx, model.xypad_value2, Msg::XYPad2Changed);
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"XYPad::new()
    .grid(true)
    .label_x("Cutoff")
    .label_y("Resonance")
    .show_with(ctx, model.filter, Msg::SetFilter);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Drag or click to set position");
            ctx.ui.label("• Double-click to reset to center");
            ctx.ui.label("• Crosshair + dot cursor");
            ctx.ui.label("• Optional grid overlay");
        }

        "Select" => {
            ctx.ui.heading("Select");
            ctx.ui.label("Dropdown select component");
            ctx.ui.add_space(16.0);

            let options = [(0, "Option A"), (1, "Option B"), (2, "Option C"), (3, "Option D")];
            Select::new(&options).show_with(ctx, Some(&model.select_value), |idx| Msg::SelectChanged(idx));

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Selected: Option {}", ["A", "B", "C", "D"][model.select_value]));
        }

        "ArcSlider" => {
            ctx.ui.heading("ArcSlider");
            ctx.ui.label("Modern arc-shaped slider for synthesizer-style controls");
            ctx.ui.add_space(16.0);

            ctx.horizontal(|ctx| {
                // Standard arc
                ctx.vertical(|ctx| {
                    ctx.ui.label("Standard:");
                    ArcSlider::new(0.0..=100.0)
                        .label("Mix")
                        .suffix("%")
                        .show_with(ctx, model.arc_value, Msg::ArcChanged);
                });

                ctx.ui.add_space(24.0);

                // Small size
                ctx.vertical(|ctx| {
                    ctx.ui.label("Small:");
                    ArcSlider::new(0.0..=1.0)
                        .small()
                        .label("Pan")
                        .show_with(ctx, model.arc_value2, Msg::Arc2Changed);
                });

                ctx.ui.add_space(24.0);

                // Large with thick arc
                ctx.vertical(|ctx| {
                    ctx.ui.label("Large + Thick:");
                    ArcSlider::new(-24.0..=24.0)
                        .large()
                        .thickness(1.5)
                        .suffix("dB")
                        .label("Gain")
                        .show_with(ctx, model.arc_value3, Msg::Arc3Changed);
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"ArcSlider::new(0.0..=100.0)
    .label("Mix")
    .suffix("%")
    .show_with(ctx, model.mix, Msg::SetMix);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Drag up/down or left/right to adjust");
            ctx.ui.label("• Double-click to reset to center");
            ctx.ui.label("• Size variants (small/medium/large)");
            ctx.ui.label("• Customizable arc thickness");
            ctx.ui.label("• Optional value suffix (%, dB, etc.)");
        }

        "ButtonGroup" => {
            ctx.ui.heading("ButtonGroup");
            ctx.ui.label("Radio-style button group with normalized 0.0-1.0 output");
            ctx.ui.add_space(16.0);

            // Wave type selector
            ctx.ui.label("Wave type (4 options → 0.0, 0.33, 0.67, 1.0):");
            ButtonGroup::new(&["Sin", "Saw", "Sqr", "Tri"])
                .show_with(ctx, model.group_value, Msg::GroupChanged);
            ctx.ui.label(format!("Value: {:.2}", model.group_value));

            ctx.ui.add_space(12.0);

            // Pan mode (3 options)
            ctx.ui.label("Pan mode (3 options → 0.0, 0.5, 1.0):");
            ButtonGroup::new(&["L", "C", "R"])
                .compact()
                .show_with(ctx, model.group_value2, Msg::Group2Changed);
            ctx.ui.label(format!("Value: {:.2}", model.group_value2));

            ctx.ui.add_space(12.0);

            // Expanded
            ctx.ui.label("Expanded to full width:");
            let mut demo_val = 0.5;
            ButtonGroup::new(&["Off", "Low", "Mid", "High", "Max"])
                .expand()
                .show(ctx.ui, &mut demo_val);

            ctx.ui.add_space(12.0);

            // Vertical
            ctx.ui.label("Vertical orientation:");
            let mut demo_val2 = 0.0;
            ButtonGroup::new(&["1", "2", "3"])
                .vertical()
                .compact()
                .show(ctx.ui, &mut demo_val2);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"ButtonGroup::new(&["Sin", "Saw", "Sqr", "Tri"])
    .show_with(ctx, model.wave_type, Msg::SetWaveType);

// Value mapping:
// 4 buttons → 0.0, 0.33, 0.67, 1.0
// 3 buttons → 0.0, 0.5, 1.0
// 2 buttons → 0.0, 1.0"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Radio-style exclusive selection");
            ctx.ui.label("• Normalized 0.0-1.0 output");
            ctx.ui.label("• Horizontal/Vertical orientation");
            ctx.ui.label("• Size variants (compact/medium/large)");
            ctx.ui.label("• Rounded outer corners only");
        }

        "Link" => {
            ctx.ui.heading("Link");
            ctx.ui.label("Hyperlink component");
            ctx.ui.add_space(8.0);

            Link::new("https://github.com", "Visit GitHub").show(ctx.ui);
            ctx.ui.add_space(4.0);
            Link::new("https://docs.rs", "Rust Docs").show(ctx.ui);
        }

        "Code" => {
            ctx.ui.heading("Code");
            ctx.ui.label("Code block with syntax highlighting");
            ctx.ui.add_space(8.0);

            Code::new("fn main() {\n    println!(\"Hello, world!\");\n}").show(ctx.ui);
        }

        "Text" => {
            ctx.ui.heading("Text");
            ctx.ui.label("Typography system with semantic variants");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Headings\nText::h1(\"Page Title\").show(ui);\nText::h2(\"Section\").show(ui);\nText::h3(\"Subsection\").show(ui);\n\n// Body variants\nText::body(\"Regular text\").show(ui);\nText::small(\"Small text\").show(ui);\nText::caption(\"Caption\").show(ui);\n\n// Modifiers\nText::body(\"Bold\").bold().show(ui);\nText::body(\"Colored\").color(theme.state_danger).show(ui);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Headings:");
            ctx.ui.add_space(4.0);
            Text::h1("Heading 1 (font_size_3xl)").show(ctx.ui);
            Text::h2("Heading 2 (font_size_2xl)").show(ctx.ui);
            Text::h3("Heading 3 (font_size_xl)").show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Body Variants:");
            ctx.ui.add_space(4.0);
            Text::large("Large text (font_size_lg)").show(ctx.ui);
            Text::body("Body text (font_size_md) - default size").show(ctx.ui);
            Text::small("Small text (font_size_sm)").show(ctx.ui);
            Text::caption("Caption text (font_size_xs)").show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Modifiers:");
            ctx.ui.add_space(4.0);
            ctx.horizontal(|ctx| {
                Text::body("Bold").bold().show(ctx.ui);
                Text::body("Italic").italic().show(ctx.ui);
                Text::body("Underline").underline().show(ctx.ui);
                Text::body("Strikethrough").strikethrough().show(ctx.ui);
            });

            ctx.ui.add_space(8.0);
            let theme = Theme::current(ctx.ui.ctx());
            ctx.horizontal(|ctx| {
                Text::body("Primary").color(theme.primary).show(ctx.ui);
                Text::body("Success").color(theme.state_success).show(ctx.ui);
                Text::body("Warning").color(theme.state_warning).show(ctx.ui);
                Text::body("Danger").color(theme.state_danger).show(ctx.ui);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Typography Tokens (from Theme):");
            ctx.ui.add_space(4.0);
            ctx.ui.label(format!("font_size_xs: {}px", theme.font_size_xs));
            ctx.ui.label(format!("font_size_sm: {}px", theme.font_size_sm));
            ctx.ui.label(format!("font_size_md: {}px", theme.font_size_md));
            ctx.ui.label(format!("font_size_lg: {}px", theme.font_size_lg));
            ctx.ui.label(format!("font_size_xl: {}px", theme.font_size_xl));
            ctx.ui.label(format!("font_size_2xl: {}px", theme.font_size_2xl));
            ctx.ui.label(format!("font_size_3xl: {}px", theme.font_size_3xl));
        }

        "Tooltip" => {
            ctx.ui.heading("Tooltip");
            ctx.ui.label("Themed tooltips via ResponseExt trait");
            ctx.ui.add_space(8.0);

            Code::new(
                "use egui_cha_ds::prelude::*;  // imports ResponseExt\n\n// Works with any egui widget\nButton::primary(\"Hover me\").show(ui).with_tooltip(\"Save changes\");\nui.button(\"Native\").with_tooltip(\"Native egui button\");"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("With DS Components:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                ctx.ui.add(Button::primary("Save")).with_tooltip("Save current document");
                ctx.ui.add(Button::secondary("Edit")).with_tooltip("Edit selected item");
                ctx.ui.add(Button::danger("Delete")).with_tooltip("Delete permanently");
            });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("With Icon-only buttons:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Icon::house().size(24.0).show(ctx.ui).with_tooltip("Go to Home");
                Icon::gear().size(24.0).show(ctx.ui).with_tooltip("Open Settings");
                Icon::user().size(24.0).show(ctx.ui).with_tooltip("View Profile");
                Icon::info().size(24.0).show(ctx.ui).with_tooltip("More Information");
            });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("With native egui widgets:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                ctx.ui.button("Native Button").with_tooltip("This works too!");
                ctx.ui.label("Hover label").with_tooltip("Labels can have tooltips");
            });
        }

        "Context Menu" => {
            ctx.ui.heading("Context Menu");
            ctx.ui.label("Right-click menu via ContextMenuExt trait");
            ctx.ui.add_space(8.0);

            Code::new(
                "use egui_cha_ds::prelude::*;  // imports ContextMenuExt\n\n// Works with any egui Response\nButton::primary(\"Right click me\")\n    .show(ctx.ui)\n    .with_context_menu(ctx, [\n        ContextMenuItem::new(\"Edit\", Msg::Edit),\n        ContextMenuItem::new(\"Copy\", Msg::Copy),\n        ContextMenuItem::separator(),\n        ContextMenuItem::danger(\"Delete\", Msg::Delete),\n    ]);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("With Button:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                ctx.ui.add(Button::primary("Right click me"))
                    .with_context_menu(ctx, [
                        ContextMenuItem::new("Edit", Msg::ContextMenuEdit),
                        ContextMenuItem::new("Copy", Msg::ContextMenuCopy),
                        ContextMenuItem::separator(),
                        ContextMenuItem::danger("Delete", Msg::ContextMenuDelete),
                    ]);
            });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("With Label:");
            ctx.ui.add_space(8.0);

            ctx.ui.label("Right click this text")
                .with_context_menu(ctx, [
                    ContextMenuItem::new("Copy text", Msg::ContextMenuCopy),
                ]);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("With Icon:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Icon::gear().size(32.0).show(ctx.ui)
                    .with_context_menu(ctx, [
                        ContextMenuItem::new("Settings", Msg::ContextMenuEdit),
                        ContextMenuItem::separator(),
                        ContextMenuItem::danger("Reset", Msg::ContextMenuDelete),
                    ]);
            });

            if let Some(action) = model.context_menu_last_action {
                ctx.ui.add_space(16.0);
                ctx.ui.label(format!("Last action: {}", action));
                Badge::success(action).show(ctx.ui);
            }
        }

        "ListItem" => {
            ctx.ui.heading("ListItem");
            ctx.ui.label("Selectable list item with optional icon and badge");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Basic usage\nListItem::new(\"Item\").on_click(ctx, Msg::Select);\n\n// With icon and selection\nListItem::new(\"Settings\")\n    .icon(icons::GEAR)\n    .selected(is_selected)\n    .on_click(ctx, Msg::Select);\n\n// With badge\nListItem::new(\"Messages\").badge(\"5\").on_click(ctx, Msg::Go);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Basic Items:");
            ctx.ui.add_space(8.0);

            ListItem::new("First item").on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Second item").on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Third item").on_click(ctx, Msg::ButtonClicked);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("With Icons:");
            ctx.ui.add_space(8.0);

            ListItem::new("Home").icon(icons::HOUSE).on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Settings").icon(icons::GEAR).on_click(ctx, Msg::ButtonClicked);
            ListItem::new("User").icon(icons::USER).on_click(ctx, Msg::ButtonClicked);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Selection State:");
            ctx.ui.add_space(8.0);

            ListItem::new("Selected item").icon(icons::CHECK).selected(true).on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Normal item").icon(icons::HASH).selected(false).on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Disabled item").icon(icons::X).disabled(true).on_click(ctx, Msg::ButtonClicked);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("With Badges:");
            ctx.ui.add_space(8.0);

            ListItem::new("Inbox").icon(icons::HASH).badge("12").on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Notifications").icon(icons::INFO).badge("3").on_click(ctx, Msg::ButtonClicked);
            ListItem::new("Updates").icon(icons::ARROW_RIGHT).badge("New").on_click(ctx, Msg::ButtonClicked);
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_audio_atom(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match AUDIO_ATOMS[model.active_component] {
        "Waveform" => {
            ctx.ui.heading("Waveform");
            ctx.ui.label("Audio waveform visualization for EDM/VJ applications");
            ctx.ui.add_space(16.0);

            let time = ctx.ui.input(|i| i.time) as f32;
            let samples: Vec<f32> = (0..128)
                .map(|i| {
                    let t = i as f32 / 128.0 * std::f32::consts::PI * 4.0 + time * 2.0;
                    (t.sin() * 0.6 + (t * 2.0).sin() * 0.3 + (t * 3.0).sin() * 0.1)
                })
                .collect();

            ctx.ui.label("Line style:");
            Waveform::new(&samples).height(60.0).show(ctx.ui);
            ctx.ui.add_space(12.0);

            ctx.ui.label("Filled style:");
            Waveform::new(&samples).height(60.0).filled().show(ctx.ui);
            ctx.ui.add_space(12.0);

            ctx.ui.label("Bars style:");
            Waveform::new(&samples).height(60.0).bars().show(ctx.ui);
        }

        "Spectrum" => {
            ctx.ui.heading("Spectrum");
            ctx.ui.label("Frequency spectrum analyzer");
            ctx.ui.add_space(16.0);

            let time = ctx.ui.input(|i| i.time) as f32;
            let fft_bins: Vec<f32> = (0..64)
                .map(|i| {
                    let freq = i as f32 / 64.0;
                    let base = (1.0 - freq).powf(1.5);
                    let pulse = (time * 2.0 + i as f32 * 0.1).sin() * 0.5 + 0.5;
                    (base * pulse * 0.8).clamp(0.0, 1.0)
                })
                .collect();

            Spectrum::new(&fft_bins).height(80.0).show(ctx.ui);
        }

        "LevelMeter" => {
            ctx.ui.heading("LevelMeter");
            ctx.ui.label("Audio level meter with peak hold");
            ctx.ui.add_space(16.0);

            let time = ctx.ui.input(|i| i.time) as f32;
            let level_db = -60.0 + (time * 1.5).sin() * 30.0 + 30.0;

            ctx.horizontal(|ctx| {
                LevelMeter::new().size(20.0, 120.0).show(ctx.ui, level_db);
                ctx.ui.add_space(8.0);
                LevelMeter::new().size(20.0, 120.0).show(ctx.ui, level_db - 6.0);
            });

            ctx.ui.ctx().request_repaint();
        }

        "Oscilloscope" => {
            ctx.ui.heading("Oscilloscope");
            ctx.ui.label("Real-time signal visualization");
            ctx.ui.add_space(16.0);

            let time = ctx.ui.input(|i| i.time) as f32;
            let samples: Vec<f32> = (0..256)
                .map(|i| {
                    let t = i as f32 / 256.0 * std::f32::consts::PI * 8.0 + time * 4.0;
                    t.sin()
                })
                .collect();

            Oscilloscope::new(&samples).height(100.0).show(ctx.ui);

            ctx.ui.ctx().request_repaint();
        }

        "BpmDisplay" => {
            ctx.ui.heading("BpmDisplay");
            ctx.ui.label("Large BPM display for DJ/VJ applications");
            ctx.ui.add_space(16.0);

            BpmDisplay::new().show(ctx.ui, 128.0);
        }

        "Transport" => {
            ctx.ui.heading("Transport");
            ctx.ui.label("Transport controls (Play/Pause/Stop/Record)");
            ctx.ui.add_space(16.0);
            ctx.ui.label("TODO: Implement TransportBar demo");
        }

        "BeatSync" => {
            ctx.ui.heading("BeatSync");
            ctx.ui.label("Beat synchronization and tap tempo");
            ctx.ui.add_space(16.0);
            ctx.ui.label("TODO: Implement BeatSync demo");
        }

        "StepSeq" => {
            ctx.ui.heading("StepSeq");
            ctx.ui.label("Step sequencer for pattern creation");
            ctx.ui.add_space(16.0);
            ctx.ui.label("TODO: Implement StepSeq demo");
        }

        "SamplePad" => {
            ctx.ui.heading("SamplePad");
            ctx.ui.label("Sample trigger pads");
            ctx.ui.add_space(16.0);
            ctx.ui.label("TODO: Implement SamplePad demo");
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_midi_atom(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match MIDI_ATOMS[model.active_component] {
        "MidiKeyboard" => {
            ctx.ui.heading("MidiKeyboard");
            ctx.ui.label("Interactive MIDI keyboard with note visualization");
            ctx.ui.add_space(8.0);

            Code::new(
                "MidiKeyboard::new()\n    .octaves(2)\n    .start_octave(4)\n    .active_notes(&model.notes)\n    .show_with(ctx, |event| match event {\n        KeyboardEvent::NoteOn(note, vel) => Msg::NoteOn(note, vel),\n        KeyboardEvent::NoteOff(note) => Msg::NoteOff(note),\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("2 Octaves (click to trigger):");
            ctx.ui.add_space(8.0);

            MidiKeyboard::new()
                .octaves(2)
                .start_octave(4)
                .active_notes(&model.keyboard_notes)
                .show_with(ctx, |event| match event {
                    KeyboardEvent::NoteOn(note, vel) => Msg::KeyboardNoteOn(note, vel),
                    KeyboardEvent::NoteOff(note) => Msg::KeyboardNoteOff(note),
                });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("3 Octaves, compact:");
            ctx.ui.add_space(8.0);

            MidiKeyboard::new()
                .octaves(3)
                .start_octave(3)
                .key_size(18.0, 60.0)
                .active_notes(&model.keyboard_notes)
                .show_with(ctx, |event| match event {
                    KeyboardEvent::NoteOn(note, vel) => Msg::KeyboardNoteOn(note, vel),
                    KeyboardEvent::NoteOff(note) => Msg::KeyboardNoteOff(note),
                });

            if !model.keyboard_notes.is_empty() {
                ctx.ui.add_space(12.0);
                ctx.ui.label(format!("Active notes: {:?}", model.keyboard_notes.iter().map(|n| n.note).collect::<Vec<_>>()));
            }
        }

        "MidiMonitor" => {
            ctx.ui.heading("MidiMonitor");
            ctx.ui.label("MIDI activity and CC state display");
            ctx.ui.add_space(8.0);

            Code::new(
                "MidiMonitor::new()\n    .device_name(\"Arturia KeyLab\")\n    .cc_values(&model.cc_state)\n    .messages(&model.midi_log)\n    .mode(MonitorMode::Split)\n    .show(ui);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Create demo CC values
            let time = ctx.ui.input(|i| i.time) as f32;
            let cc_values = vec![
                CcValue::new(1, ((time * 0.5).sin() * 63.5 + 63.5) as u8).with_label("Mod"),
                CcValue::new(7, 100).with_label("Vol"),
                CcValue::new(10, 64).with_label("Pan"),
                CcValue::new(74, ((time * 0.3).cos() * 63.5 + 63.5) as u8).with_label("Cutoff"),
            ];

            // Create demo messages
            let messages = vec![
                MidiMessage::NoteOn(0, 60, 100),
                MidiMessage::ControlChange(0, 1, 64),
                MidiMessage::NoteOff(0, 60),
                MidiMessage::PitchBend(0, 0),
            ];

            ctx.ui.strong("CC Grid mode:");
            ctx.ui.add_space(8.0);

            MidiMonitor::new()
                .device_name("Demo Controller")
                .cc_values(&cc_values)
                .mode(MonitorMode::CcGrid)
                .size(300.0, 120.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Message Log mode:");
            ctx.ui.add_space(8.0);

            MidiMonitor::new()
                .device_name("Demo Controller")
                .messages(&messages)
                .mode(MonitorMode::MessageLog)
                .size(300.0, 120.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Split mode (CC + Log):");
            ctx.ui.add_space(8.0);

            MidiMonitor::new()
                .device_name("Demo Controller")
                .cc_values(&cc_values)
                .messages(&messages)
                .mode(MonitorMode::Split)
                .size(400.0, 150.0)
                .show(ctx.ui);
        }

        "MidiMapper" => {
            ctx.ui.heading("MidiMapper");
            ctx.ui.label("MIDI CC/note parameter mapping with learn mode");
            ctx.ui.add_space(8.0);

            Code::new(
                "MidiMapper::new(&params, &mappings)\n    .learn_state(&model.learn_state)\n    .show_with(ctx, |event| match event {\n        MidiMapperEvent::StartLearn(id) => Msg::StartLearn(id),\n        MidiMapperEvent::AssignMapping(m) => Msg::Assign(m),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Create demo params
            let params = vec![
                MappableParam::new("volume", "Master Volume").with_group("Output").with_value(0.75),
                MappableParam::new("pan", "Pan").with_group("Output").with_value(0.5),
                MappableParam::new("cutoff", "Filter Cutoff").with_group("Filter").with_value(0.6),
                MappableParam::new("resonance", "Resonance").with_group("Filter").with_value(0.3),
            ];

            // Demo mappings
            let mappings = vec![
                MidiMapping::new("volume", MidiMsgType::CC, 0, 7),
                MidiMapping::new("cutoff", MidiMsgType::CC, 0, 74),
            ];

            MidiMapper::new(&params, &mappings)
                .size(380.0, 200.0)
                .show_values(true)
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Click 'Learn' to enter MIDI learn mode");
        }

        "PianoRoll" => {
            ctx.ui.heading("PianoRoll");
            ctx.ui.label("MIDI note editor with keyboard, grid, and playhead");
            ctx.ui.add_space(8.0);

            Code::new(
                "PianoRoll::new()\n    .notes(&model.notes)\n    .position(model.playhead)\n    .bars(4)\n    .show_with(ctx, |event| match event {\n        PianoRollEvent::NoteAdd(n, s, d) => Msg::AddNote(n, s, d),\n        PianoRollEvent::Seek(pos) => Msg::Seek(pos),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Create demo notes if empty
            let demo_notes: Vec<MidiNote> = if model.piano_notes.is_empty() {
                vec![
                    MidiNote::new(60, 0.0, 1.0).with_velocity(100),
                    MidiNote::new(64, 1.0, 1.0).with_velocity(80),
                    MidiNote::new(67, 2.0, 2.0).with_velocity(90),
                    MidiNote::new(72, 4.0, 1.0).with_velocity(100),
                    MidiNote::new(71, 5.0, 0.5).with_velocity(70),
                    MidiNote::new(69, 5.5, 0.5).with_velocity(70),
                    MidiNote::new(67, 6.0, 2.0).with_velocity(85),
                ]
            } else {
                model.piano_notes.clone()
            };

            // Animate playhead
            let time = ctx.ui.input(|i| i.time) as f32;
            let position = (time * 0.5) % 8.0; // Loop over 8 beats

            ctx.ui.strong("4 bars, editable:");
            ctx.ui.add_space(8.0);

            PianoRoll::new()
                .notes(&demo_notes)
                .position(position)
                .bars(2)
                .note_range(48, 84)
                .selected(model.piano_selected)
                .show_with(ctx, |event| match event {
                    PianoRollEvent::NoteAdd(n, s, d) => Msg::PianoNoteAdd(n, s, d),
                    PianoRollEvent::NoteMove(i, n, s) => Msg::PianoNoteMove(i, n, s),
                    PianoRollEvent::NoteDelete(i) => Msg::PianoNoteDelete(i),
                    PianoRollEvent::NoteSelect(i) => Msg::PianoNoteSelect(i),
                    PianoRollEvent::Seek(pos) => Msg::PianoSeek(pos),
                    _ => Msg::PianoSeek(0.0),
                });

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Double-click to add notes");
            ctx.ui.label("• Drag to move notes");
            ctx.ui.label("• Right-click to delete");
            ctx.ui.label("• Click grid to seek");
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_mixer_atom(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match MIXER_ATOMS[model.active_component] {
        "ChannelStrip" => {
            ctx.ui.heading("ChannelStrip");
            ctx.ui.label("Mixer channel strip with fader, pan, mute/solo, and level meter");
            ctx.ui.add_space(8.0);

            Code::new(
                "ChannelStrip::new(\"Drums\")\n    .volume(model.volume)\n    .pan(model.pan)\n    .level(model.peak)\n    .mute(model.muted)\n    .solo(model.soloed)\n    .show_with(ctx, |e| match e {\n        ChannelEvent::VolumeChange(v) => Msg::SetVol(v),\n        ChannelEvent::Mute => Msg::ToggleMute,\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Animate levels
            let time = ctx.ui.input(|i| i.time) as f32;
            let levels = [
                ((time * 2.0).sin() * 0.3 + 0.5).clamp(0.0, 1.0),
                ((time * 1.5 + 1.0).sin() * 0.3 + 0.6).clamp(0.0, 1.0),
                ((time * 1.8 + 2.0).sin() * 0.25 + 0.45).clamp(0.0, 1.0),
                ((time * 2.2 + 3.0).sin() * 0.35 + 0.55).clamp(0.0, 1.0),
            ];

            ctx.ui.strong("4-Channel Mixer:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                let labels = ["Kick", "Snare", "Hi-Hat", "Bass"];
                let colors = [
                    egui::Color32::from_rgb(220, 80, 80),
                    egui::Color32::from_rgb(80, 180, 220),
                    egui::Color32::from_rgb(220, 180, 80),
                    egui::Color32::from_rgb(120, 220, 120),
                ];

                for i in 0..4 {
                    ChannelStrip::new(labels[i])
                        .volume(model.channel_volumes[i])
                        .pan(model.channel_pans[i])
                        .level(levels[i])
                        .mute(model.channel_mutes[i])
                        .solo(model.channel_solos[i])
                        .color(colors[i])
                        .width(55.0)
                        .compact(true)
                        .show_with(ctx, |e| match e {
                            ChannelEvent::VolumeChange(v) => Msg::ChannelVolume(i, v),
                            ChannelEvent::PanChange(p) => Msg::ChannelPan(i, p),
                            ChannelEvent::Mute => Msg::ChannelMute(i),
                            ChannelEvent::Solo => Msg::ChannelSolo(i),
                            _ => Msg::ButtonClicked,
                        });

                    ctx.ui.add_space(4.0);
                }
            });
        }

        "CrossFader" => {
            ctx.ui.heading("CrossFader");
            ctx.ui.label("DJ-style A/B crossfader with curve options");
            ctx.ui.add_space(8.0);

            Code::new(
                "CrossFader::new()\n    .value(model.mix)  // -1.0 to 1.0\n    .labels(\"Deck A\", \"Deck B\")\n    .curve(CrossfaderCurve::EqualPower)\n    .show_with(ctx, Msg::SetMix);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Linear curve:");
            ctx.ui.add_space(8.0);

            CrossFader::new()
                .value(model.crossfader_value)
                .labels("Deck A", "Deck B")
                .curve(CrossfaderCurve::Linear)
                .size(300.0, 40.0)
                .show_with(ctx, Msg::CrossfaderChange);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Equal Power curve (constant loudness):");
            ctx.ui.add_space(8.0);

            CrossFader::new()
                .value(model.crossfader_value)
                .labels("Source A", "Source B")
                .curve(CrossfaderCurve::EqualPower)
                .color_a(egui::Color32::from_rgb(100, 180, 255))
                .color_b(egui::Color32::from_rgb(255, 150, 100))
                .size(300.0, 40.0)
                .show_with(ctx, Msg::CrossfaderChange);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Fast Cut (scratch style):");
            ctx.ui.add_space(8.0);

            CrossFader::new()
                .value(model.crossfader_value)
                .curve(CrossfaderCurve::FastCut)
                .size(300.0, 40.0)
                .show_with(ctx, Msg::CrossfaderChange);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Position: {:.2}", model.crossfader_value));
        }

        "EffectRack" => {
            ctx.ui.heading("EffectRack");
            ctx.ui.label("Audio effect chain display and control");
            ctx.ui.add_space(8.0);

            Code::new(
                "EffectRack::new(&model.effects)\n    .show_with(ctx, |e| match e {\n        RackEvent::Toggle(i) => Msg::ToggleEffect(i),\n        RackEvent::Reorder(from, to) => Msg::Reorder(from, to),\n        RackEvent::Select(i) => Msg::SelectEffect(i),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Create demo effects
            let effects = vec![
                Effect::new("Compressor", EffectCategory::Dynamics)
                    .enabled(true)
                    .with_param(EffectParam::new("Threshold", 0.6).with_range(-60.0, 0.0))
                    .with_param(EffectParam::new("Ratio", 0.4).with_range(1.0, 20.0)),
                Effect::new("EQ Eight", EffectCategory::EQ)
                    .enabled(true),
                Effect::new("Reverb", EffectCategory::Time)
                    .enabled(false)
                    .with_param(EffectParam::new("Size", 0.5))
                    .with_param(EffectParam::new("Decay", 0.7)),
            ];

            EffectRack::new(&effects)
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Click effect to select");
            ctx.ui.label("• Toggle bypass with power button");
        }

        "EnvelopeEditor" => {
            ctx.ui.heading("EnvelopeEditor");
            ctx.ui.label("ADSR envelope and custom curve editor");
            ctx.ui.add_space(8.0);

            Code::new(
                "EnvelopeEditor::adsr()\n    .attack(0.1).decay(0.2)\n    .sustain(0.7).release(0.3)\n    .show_with(ctx, |e| match e {\n        EnvelopeEvent::AttackChange(v) => Msg::SetA(v),\n        EnvelopeEvent::SustainChange(v) => Msg::SetS(v),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("ADSR Envelope:");
            ctx.ui.add_space(8.0);

            EnvelopeEditor::adsr()
                .attack(0.1)
                .decay(0.2)
                .sustain(0.7)
                .release(0.4)
                .size(280.0, 120.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Custom envelope:");
            ctx.ui.add_space(8.0);

            let custom_points = vec![
                EnvelopePoint::new(0.0, 0.0),
                EnvelopePoint::new(0.1, 1.0).with_curve(CurveType::Exponential),
                EnvelopePoint::new(0.3, 0.6),
                EnvelopePoint::new(0.6, 0.8).with_curve(CurveType::SCurve),
                EnvelopePoint::new(1.0, 0.0),
            ];

            EnvelopeEditor::custom(&custom_points)
                .size(280.0, 100.0)
                .show(ctx.ui);
        }

        "AutomationLane" => {
            ctx.ui.heading("AutomationLane");
            ctx.ui.label("Parameter automation over time with playhead");
            ctx.ui.add_space(8.0);

            Code::new(
                "AutomationLane::new(\"Filter Cutoff\")\n    .points(&model.automation)\n    .position(model.playhead)\n    .show_with(ctx, |e| match e {\n        AutomationEvent::PointMove(i, t, v) => Msg::Move(i, t, v),\n        AutomationEvent::Seek(pos) => Msg::Seek(pos),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Animate playhead
            let time = ctx.ui.input(|i| i.time) as f32;
            let position = (time * 0.1) % 1.0;

            let automation_points = vec![
                AutomationPoint::new(0.0, 0.3),
                AutomationPoint::new(0.25, 0.8).with_curve(AutomationCurve::Smooth),
                AutomationPoint::new(0.5, 0.4),
                AutomationPoint::new(0.75, 0.9).with_curve(AutomationCurve::Exponential),
                AutomationPoint::new(1.0, 0.3),
            ];

            ctx.ui.strong("Filter Cutoff automation:");
            ctx.ui.add_space(8.0);

            AutomationLane::new("Filter Cutoff")
                .points(&automation_points)
                .position(position)
                .range(20.0..=20000.0)
                .height(80.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            let volume_points = vec![
                AutomationPoint::new(0.0, 0.7),
                AutomationPoint::new(0.3, 0.7).with_curve(AutomationCurve::Step),
                AutomationPoint::new(0.3, 0.0),
                AutomationPoint::new(0.6, 0.0).with_curve(AutomationCurve::Step),
                AutomationPoint::new(0.6, 0.9),
                AutomationPoint::new(1.0, 0.9),
            ];

            ctx.ui.strong("Volume automation (stepped):");
            ctx.ui.add_space(8.0);

            AutomationLane::new("Volume")
                .points(&volume_points)
                .position(position)
                .height(60.0)
                .color(egui::Color32::from_rgb(100, 200, 100))
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Double-click to add points");
            ctx.ui.label("• Drag points to edit");
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_visual_atom(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match VISUAL_ATOMS[model.active_component] {
        "ClipGrid" => {
            ctx.ui.heading("ClipGrid");
            ctx.ui.label("Ableton Live style clip launcher grid");
            ctx.ui.add_space(8.0);

            Code::new(
                "ClipGrid::new(&clips, 4)\n    .current(model.playing_clip)\n    .queued(&model.queued)\n    .show_with(ctx, |idx| Msg::QueueClip(idx));"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let clips = vec![
                ClipCell::new("Intro").with_color(egui::Color32::from_rgb(120, 180, 255)),
                ClipCell::new("Verse 1").with_color(egui::Color32::from_rgb(180, 255, 120)),
                ClipCell::new("Chorus").with_color(egui::Color32::from_rgb(255, 180, 120)).with_state(ClipState::Playing),
                ClipCell::new("Break").with_color(egui::Color32::from_rgb(255, 120, 180)),
                ClipCell::new("Verse 2").with_color(egui::Color32::from_rgb(180, 255, 120)),
                ClipCell::new("Chorus 2").with_color(egui::Color32::from_rgb(255, 180, 120)).with_state(ClipState::Queued),
                ClipCell::new("Outro").with_color(egui::Color32::from_rgb(200, 200, 200)),
                ClipCell::new("Alt End").with_color(egui::Color32::from_rgb(150, 150, 200)),
            ];

            ctx.ui.strong("4x2 Clip Grid:");
            ctx.ui.add_space(8.0);

            ClipGrid::new(&clips, 4)
                .current(Some(2))
                .queued(&[5])
                .cell_size(80.0, 50.0)
                .show_index(true)
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Green border = Playing");
            ctx.ui.label("• Pulsing = Queued");
        }

        "Timeline" => {
            ctx.ui.heading("Timeline");
            ctx.ui.label("Seek bar with markers, regions, and time display");
            ctx.ui.add_space(8.0);

            Code::new(
                "Timeline::new(120.0)  // 2 minutes\n    .position(model.position)\n    .markers(&model.markers)\n    .show_with(ctx, |e| match e {\n        TimelineEvent::Seek(p) => Msg::Seek(p),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Animate position
            let time = ctx.ui.input(|i| i.time);
            let position = (time * 0.05) % 1.0;

            let markers = vec![
                TimelineMarker::new(0.0, "Start"),
                TimelineMarker::new(0.25, "Verse").with_color(egui::Color32::from_rgb(100, 200, 100)),
                TimelineMarker::new(0.5, "Chorus").with_color(egui::Color32::from_rgb(200, 100, 200)),
                TimelineMarker::new(0.75, "Bridge").with_color(egui::Color32::from_rgb(200, 200, 100)),
            ];

            let regions = vec![
                TimelineRegion::new(0.25, 0.5, egui::Color32::from_rgba_unmultiplied(100, 200, 100, 40)),
                TimelineRegion::new(0.5, 0.75, egui::Color32::from_rgba_unmultiplied(200, 100, 200, 40)),
            ];

            ctx.ui.strong("Basic timeline (2 min duration):");
            ctx.ui.add_space(8.0);

            Timeline::new(120.0)
                .position(position)
                .markers(&markers)
                .height(36.0)
                .show_with(ctx, |e| match e {
                    TimelineEvent::Seek(p) => Msg::TimelineSeek(p),
                    _ => Msg::TimelineSeek(0.0),
                });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("With regions and loop:");
            ctx.ui.add_space(8.0);

            Timeline::new(120.0)
                .position(position)
                .markers(&markers)
                .regions(&regions)
                .loop_region(0.25, 0.75)
                .height(36.0)
                .show_with(ctx, |e| match e {
                    TimelineEvent::Seek(p) => Msg::TimelineSeek(p),
                    _ => Msg::TimelineSeek(0.0),
                });
        }

        "Preview" => {
            ctx.ui.heading("Preview");
            ctx.ui.label("Video/image preview with overlays");
            ctx.ui.add_space(8.0);

            Code::new(
                "Preview::new(texture_id)\n    .size(320.0, 180.0)\n    .timecode(\"00:01:23:15\")\n    .state(PreviewState::Playing)\n    .label(\"Clip A\")\n    .show_with(ctx, |e| match e {\n        PreviewEvent::Click => Msg::TogglePlay,\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Simulate timecode
            let time = ctx.ui.input(|i| i.time);
            let frames = (time * 30.0) as u32;
            let secs = (frames / 30) % 60;
            let mins = (frames / 30 / 60) % 60;
            let frame = frames % 30;
            let timecode = format!("00:{:02}:{:02}:{:02}", mins, secs, frame);

            ctx.horizontal(|ctx| {
                ctx.ui.strong("Playing:");
                ctx.ui.add_space(8.0);

                Preview::empty()
                    .size(200.0, 112.0)
                    .aspect_ratio(AspectRatio::Widescreen)
                    .timecode(&timecode)
                    .state(PreviewState::Playing)
                    .label("Main Output")
                    .show(ctx.ui);

                ctx.ui.add_space(16.0);

                ctx.ui.strong("Paused:");
                ctx.ui.add_space(8.0);

                Preview::empty()
                    .size(200.0, 112.0)
                    .timecode("00:00:42:15")
                    .state(PreviewState::Paused)
                    .label("Preview")
                    .show(ctx.ui);
            });

            ctx.ui.add_space(16.0);

            ctx.horizontal(|ctx| {
                ctx.ui.strong("Live:");
                ctx.ui.add_space(8.0);

                Preview::empty()
                    .size(150.0, 150.0)
                    .aspect_ratio(AspectRatio::Square)
                    .state(PreviewState::Live)
                    .label("Camera 1")
                    .show(ctx.ui);

                ctx.ui.add_space(16.0);

                ctx.ui.strong("Loading:");
                ctx.ui.add_space(8.0);

                Preview::empty()
                    .size(150.0, 150.0)
                    .state(PreviewState::Loading)
                    .label("Loading...")
                    .show(ctx.ui);
            });
        }

        "LayerStack" => {
            ctx.ui.heading("LayerStack");
            ctx.ui.label("Layer management with blend modes and reordering");
            ctx.ui.add_space(8.0);

            Code::new(
                "LayerStack::new(&layers)\n    .selected(model.selected)\n    .show_with(ctx, |e| match e {\n        LayerEvent::Select(i) => Msg::Select(i),\n        LayerEvent::SetOpacity(i, v) => Msg::SetOpacity(i, v),\n        LayerEvent::SetBlendMode(i, m) => Msg::SetBlend(i, m),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let layers = vec![
                Layer::new("Background").with_color(egui::Color32::from_rgb(100, 150, 200)),
                Layer::new("Video 1").with_opacity(0.8).with_blend_mode(BlendMode::Normal)
                    .with_color(egui::Color32::from_rgb(200, 150, 100)),
                Layer::new("Overlay").with_opacity(0.6).with_blend_mode(BlendMode::Add)
                    .with_color(egui::Color32::from_rgb(150, 200, 100)),
                Layer::new("Text").with_blend_mode(BlendMode::Screen)
                    .with_color(egui::Color32::from_rgb(200, 100, 150)),
            ];

            LayerStack::new(&layers)
                .selected(Some(1))
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Drag to reorder layers");
            ctx.ui.label("• Click eye icon to toggle visibility");
        }

        "ColorWheel" => {
            ctx.ui.heading("ColorWheel");
            ctx.ui.label("HSV color picker with hue ring and SV area");
            ctx.ui.add_space(8.0);

            Code::new(
                "ColorWheel::new()\n    .show_alpha(true)\n    .show_with(ctx, model.color, Msg::SetColor);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let current_color = model.wheel_color.to_color32();

            ctx.horizontal(|ctx| {
                ctx.ui.strong("Triangle style:");
                ctx.ui.add_space(8.0);

                ColorWheel::new()
                    .style(WheelStyle::Triangle)
                    .size(160.0)
                    .show_with(ctx, current_color, |c| Msg::ColorWheelChange(Hsva::from_color32(c)));

                ctx.ui.add_space(24.0);

                ctx.ui.strong("Square style:");
                ctx.ui.add_space(8.0);

                ColorWheel::new()
                    .style(WheelStyle::Square)
                    .size(160.0)
                    .show_with(ctx, current_color, |c| Msg::ColorWheelChange(Hsva::from_color32(c)));
            });

            ctx.ui.add_space(16.0);

            ctx.ui.label(format!(
                "HSV: ({:.0}°, {:.0}%, {:.0}%)  RGB: #{:02X}{:02X}{:02X}",
                model.wheel_color.h * 360.0,
                model.wheel_color.s * 100.0,
                model.wheel_color.v * 100.0,
                current_color.r(), current_color.g(), current_color.b()
            ));
        }

        "GradientEditor" => {
            ctx.ui.heading("GradientEditor");
            ctx.ui.label("Color gradient stop editor");
            ctx.ui.add_space(8.0);

            Code::new(
                "GradientEditor::new(&gradient)\n    .show_with(ctx, |e| match e {\n        GradientEvent::StopMove(i, pos) => Msg::MoveStop(i, pos),\n        GradientEvent::StopColorChange(i, c) => Msg::SetStopColor(i, c),\n        GradientEvent::StopAdd(pos, c) => Msg::AddStop(pos, c),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let gradient = Gradient::from_stops(vec![
                GradientStop::new(0.0, egui::Color32::from_rgb(255, 100, 100)),
                GradientStop::new(0.5, egui::Color32::from_rgb(100, 255, 100)),
                GradientStop::new(1.0, egui::Color32::from_rgb(100, 100, 255)),
            ]);

            ctx.ui.strong("Horizontal gradient:");
            ctx.ui.add_space(8.0);

            GradientEditor::new(&gradient)
                .direction(GradientDirection::Horizontal)
                .width(300.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            let gradient2 = Gradient::from_stops(vec![
                GradientStop::new(0.0, egui::Color32::from_rgb(50, 50, 80)),
                GradientStop::new(0.3, egui::Color32::from_rgb(100, 50, 150)),
                GradientStop::new(0.7, egui::Color32::from_rgb(200, 100, 50)),
                GradientStop::new(1.0, egui::Color32::from_rgb(255, 200, 100)),
            ]);

            ctx.ui.strong("Sunset gradient:");
            ctx.ui.add_space(8.0);

            GradientEditor::new(&gradient2)
                .width(300.0)
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Drag stops to reposition");
            ctx.ui.label("• Double-click to add stops");
        }

        "MaskEditor" => {
            ctx.ui.heading("MaskEditor");
            ctx.ui.label("Mask shape editing with points and curves");
            ctx.ui.add_space(8.0);

            Code::new(
                "MaskEditor::new(&mask)\n    .show_with(ctx, |e| match e {\n        MaskEvent::PointMove(i, p) => Msg::MoveMaskPoint(i, p),\n        MaskEvent::PointAdd(i, p) => Msg::AddMaskPoint(i, p),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            // Diamond-ish mask
            let mask = Mask::polygon(vec![
                MaskPoint::new(0.5, 0.1),
                MaskPoint::new(0.9, 0.5),
                MaskPoint::new(0.5, 0.9),
                MaskPoint::new(0.1, 0.5),
            ]);

            ctx.ui.strong("Polygon mask:");
            ctx.ui.add_space(8.0);

            MaskEditor::new(&mask)
                .size(200.0, 200.0)
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Drag points to reshape");
            ctx.ui.label("• Click edge to add points");
        }

        "TransformGizmo" => {
            ctx.ui.heading("TransformGizmo");
            ctx.ui.label("2D transform manipulation handles");
            ctx.ui.add_space(8.0);

            Code::new(
                "TransformGizmo::new()\n    .transform(model.transform)\n    .mode(TransformMode::Scale)\n    .show_with(ctx, |e| match e {\n        TransformEvent::Move(dx, dy) => Msg::Move(dx, dy),\n        TransformEvent::Scale(sx, sy) => Msg::Scale(sx, sy),\n        TransformEvent::Rotate(angle) => Msg::Rotate(angle),\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let transform = Transform2D::new()
                .with_position(egui::vec2(100.0, 75.0))
                .with_scale(egui::vec2(160.0, 90.0));

            ctx.ui.strong("All transforms mode:");
            ctx.ui.add_space(8.0);

            TransformGizmo::new()
                .size(300.0, 200.0)
                .mode(TransformMode::All)
                .show(ctx.ui, &transform);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Drag center to move");
            ctx.ui.label("• Drag corners to scale");
            ctx.ui.label("• Drag outside to rotate");
        }

        "MediaBrowser" => {
            ctx.ui.heading("MediaBrowser");
            ctx.ui.label("Thumbnail grid for media selection");
            ctx.ui.add_space(8.0);

            Code::new(
                "MediaBrowser::new(&items)\n    .selected(model.selected_media)\n    .view_mode(BrowserViewMode::Grid)\n    .show_with(ctx, |e| match e {\n        MediaBrowserEvent::Select(i) => Msg::SelectMedia(i),\n        MediaBrowserEvent::DoubleClick(i) => Msg::LoadMedia(i),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let items = vec![
                MediaItem::new("1", "clip_001.mov").with_type(MediaType::Video),
                MediaItem::new("2", "background.jpg").with_type(MediaType::Image),
                MediaItem::new("3", "overlay.png").with_type(MediaType::Image),
                MediaItem::new("4", "loop_beat.wav").with_type(MediaType::Audio),
                MediaItem::new("5", "intro.mov").with_type(MediaType::Video),
                MediaItem::new("6", "outro.mov").with_type(MediaType::Video),
            ];

            ctx.ui.strong("Grid view:");
            ctx.ui.add_space(8.0);

            MediaBrowser::new(&items)
                .selected(Some("1"))
                .view_mode(BrowserViewMode::Grid)
                .size(350.0, 150.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("List view:");
            ctx.ui.add_space(8.0);

            MediaBrowser::new(&items)
                .selected(Some("2"))
                .view_mode(BrowserViewMode::List)
                .size(350.0, 120.0)
                .show(ctx.ui);
        }

        "OutputRouter" => {
            ctx.ui.heading("OutputRouter");
            ctx.ui.label("Multi-output routing matrix");
            ctx.ui.add_space(8.0);

            Code::new(
                "OutputRouter::new(&sources, &outputs)\n    .connections(&model.routes)\n    .show_with(ctx, |e| match e {\n        RouterEvent::Connect(src, out) => Msg::Connect(src, out),\n        RouterEvent::Disconnect(src, out) => Msg::Disconnect(src, out),\n        _ => Msg::Noop,\n    });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            let sources = vec![
                RouteSource::new("main", "Main Mix").with_type(SourceType::Main),
                RouteSource::new("cam1", "Camera 1").with_type(SourceType::Aux),
                RouteSource::new("gfx", "Graphics").with_type(SourceType::Layer),
            ];

            let outputs = vec![
                RouteOutput::new("proj1", "Projector 1").with_type(OutputType::Display),
                RouteOutput::new("led", "LED Wall").with_type(OutputType::Display),
                RouteOutput::new("stream", "Stream").with_type(OutputType::Stream),
                RouteOutput::new("record", "Record").with_type(OutputType::Record),
            ];

            let connections = vec![
                RouteConnection::new("main", "proj1"),  // Main Mix -> Projector 1
                RouteConnection::new("main", "stream"), // Main Mix -> Stream
                RouteConnection::new("cam1", "led"),    // Camera 1 -> LED Wall
            ];

            OutputRouter::new(&sources, &outputs, &connections)
                .size(380.0, 180.0)
                .show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("• Click intersections to toggle routing");
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_semantics(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match SEMANTICS[model.active_component] {
        "Overview" => {
            ctx.ui.heading("Semantic Buttons");
            ctx.ui.add_space(4.0);
            ctx.ui.label("Buttons with fixed labels and icons for UI consistency.");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Atoms: style only, label is your choice\nButton::primary(\"Save\").on_click(ctx, Msg::Save);\nButton::primary(\"保存\").on_click(ctx, Msg::Save);  // inconsistent!\n\n// Semantics: label & icon fixed by framework\nsemantics::save(ButtonStyle::Both).on_click(ctx, Msg::Save);  // Always \"Save\""
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Why Semantics?");
            ctx.ui.label("• Prevents label inconsistency (Save vs 保存 vs SAVE)");
            ctx.ui.label("• Icon + color automatically matched to action");
            ctx.ui.label("• Only display style (Icon/Text/Both) is configurable");

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("All Semantic Buttons:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                semantics::save(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::edit(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::delete(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::close(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
            ctx.horizontal(|ctx| {
                semantics::add(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::remove(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::search(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::refresh(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
            ctx.horizontal(|ctx| {
                semantics::play(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::pause(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::stop(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::settings(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
            ctx.horizontal(|ctx| {
                semantics::back(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::forward(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::confirm(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::cancel(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
            ctx.horizontal(|ctx| {
                semantics::copy(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
        }

        "File Operations" => {
            ctx.ui.heading("File Operations");
            ctx.ui.label("Save, Edit, Delete, Close");
            ctx.ui.add_space(8.0);

            ctx.ui.strong("save() - Primary style");
            ctx.horizontal(|ctx| {
                semantics::save(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::save(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::save(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("edit() - Primary style");
            ctx.horizontal(|ctx| {
                semantics::edit(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::edit(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::edit(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("delete() - Danger style");
            ctx.horizontal(|ctx| {
                semantics::delete(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::delete(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::delete(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("close() - Secondary style");
            ctx.horizontal(|ctx| {
                semantics::close(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::close(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::close(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
        }

        "Actions" => {
            ctx.ui.heading("Common Actions");
            ctx.ui.label("Add, Remove, Search, Refresh, Settings, Copy");
            ctx.ui.add_space(8.0);

            ctx.ui.strong("add() - Primary");
            ctx.horizontal(|ctx| {
                semantics::add(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::add(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::add(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("remove() - Danger");
            ctx.horizontal(|ctx| {
                semantics::remove(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::remove(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::remove(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("search() - Primary");
            ctx.horizontal(|ctx| {
                semantics::search(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::search(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::search(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("refresh() - Secondary");
            ctx.horizontal(|ctx| {
                semantics::refresh(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::refresh(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::refresh(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("settings() - Secondary");
            ctx.horizontal(|ctx| {
                semantics::settings(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::settings(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::settings(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("copy() - Secondary");
            ctx.horizontal(|ctx| {
                semantics::copy(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::copy(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::copy(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
        }

        "Media" => {
            ctx.ui.heading("Media Controls");
            ctx.ui.label("Play, Pause, Stop");
            ctx.ui.add_space(8.0);

            ctx.ui.strong("play() - Success style");
            ctx.horizontal(|ctx| {
                semantics::play(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::play(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::play(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("pause() - Secondary style");
            ctx.horizontal(|ctx| {
                semantics::pause(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::pause(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::pause(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("stop() - Danger style");
            ctx.horizontal(|ctx| {
                semantics::stop(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::stop(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::stop(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Example: Media Player Controls");
            ctx.horizontal(|ctx| {
                semantics::back(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::play(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::pause(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::stop(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::forward(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
            });
        }

        "Navigation" => {
            ctx.ui.heading("Navigation & Confirmation");
            ctx.ui.label("Back, Forward, Confirm, Cancel");
            ctx.ui.add_space(8.0);

            ctx.ui.strong("back() - Secondary");
            ctx.horizontal(|ctx| {
                semantics::back(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::back(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::back(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("forward() - Secondary");
            ctx.horizontal(|ctx| {
                semantics::forward(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::forward(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::forward(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("confirm() - Success");
            ctx.horizontal(|ctx| {
                semantics::confirm(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::confirm(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::confirm(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("cancel() - Secondary");
            ctx.horizontal(|ctx| {
                semantics::cancel(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::cancel(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::cancel(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Example: Dialog Actions");
            ctx.horizontal(|ctx| {
                semantics::cancel(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::confirm(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
        }

        "ButtonStyle" => {
            ctx.ui.heading("ButtonStyle Enum");
            ctx.ui.label("Controls how semantic buttons are displayed");
            ctx.ui.add_space(8.0);

            Code::new(
                "pub enum ButtonStyle {\n    Icon,  // Icon only (compact)\n    Text,  // Text label only\n    Both,  // Icon + Text (most explicit)\n}"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("ButtonStyle::Icon");
            ctx.ui.label("Compact, icon-only. Good for toolbars.");
            ctx.horizontal(|ctx| {
                semantics::save(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::edit(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::delete(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
                semantics::refresh(ButtonStyle::Icon).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("ButtonStyle::Text");
            ctx.ui.label("Text-only. Good for menus or when icons aren't needed.");
            ctx.horizontal(|ctx| {
                semantics::save(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::edit(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::delete(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
                semantics::refresh(ButtonStyle::Text).on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(12.0);
            ctx.ui.strong("ButtonStyle::Both");
            ctx.ui.label("Icon + Text. Most explicit, good for primary actions.");
            ctx.horizontal(|ctx| {
                semantics::save(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::edit(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::delete(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
                semantics::refresh(ButtonStyle::Both).on_click(ctx, Msg::ButtonClicked);
            });
        }

        "SeverityLog" => {
            ctx.ui.heading("SeverityLog");
            ctx.ui.label("Severity-based log display using Theme's log_* colors");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Simple log messages\nSeverityLog::debug(\"Debug message\").show(ui);\nSeverityLog::info(\"Info message\").show(ui);\nSeverityLog::warn(\"Warning message\").show(ui);\nSeverityLog::error(\"Error message\").show(ui);\nSeverityLog::critical(\"Critical message\").show(ui);\n\n// With label\nSeverityLog::error(\"With label\").with_label(true).show(ui);\n\n// Framed style\nSeverityLog::error(\"Framed\").show_framed(ui);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("All Severity Levels:");
            ctx.ui.add_space(8.0);

            SeverityLog::debug("Debug: verbose diagnostic info").show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::info("Info: general information").show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::warn("Warn: something might be wrong").show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::error("Error: something went wrong").show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::critical("Critical: system failure!").show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("With Labels:");
            ctx.ui.add_space(8.0);

            SeverityLog::debug("Debug message").with_label(true).show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::info("Info message").with_label(true).show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::warn("Warning message").with_label(true).show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::error("Error message").with_label(true).show(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::critical("Critical message").with_label(true).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Framed Style:");
            ctx.ui.add_space(8.0);

            SeverityLog::debug("Debug with background").show_framed(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::info("Info with background").show_framed(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::warn("Warning with background").show_framed(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::error("Error with background").show_framed(ctx.ui);
            ctx.ui.add_space(4.0);
            SeverityLog::critical("Critical with background").show_framed(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Icon Only (no label):");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                SeverityLog::debug("").with_label(false).show(ctx.ui);
                SeverityLog::info("").with_label(false).show(ctx.ui);
                SeverityLog::warn("").with_label(false).show(ctx.ui);
                SeverityLog::error("").with_label(false).show(ctx.ui);
                SeverityLog::critical("").with_label(false).show(ctx.ui);
            });
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_molecule(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match MOLECULES[model.active_component] {
        "Card" => {
            ctx.ui.heading("Card");
            ctx.ui.label("Container with optional title");
            ctx.ui.add_space(8.0);

            Card::titled("Card Title").show_ctx(ctx, |ctx| {
                ctx.ui.label("This is card content.");
                ctx.ui.label("Cards group related information.");
            });

            ctx.ui.add_space(8.0);

            Card::new().show_ctx(ctx, |ctx| {
                ctx.ui.label("Card without title");
            });
        }

        "Tabs" => {
            ctx.ui.heading("Tabs");
            ctx.ui.label("Tabbed navigation");
            ctx.ui.add_space(8.0);

            Tabs::new(&["First", "Second", "Third"])
                .show_with(ctx, model.tabs_index, Msg::TabChanged);

            ctx.ui.add_space(8.0);

            TabPanel::show_ctx(ctx, model.tabs_index, 0, |ctx| {
                ctx.ui.label("Content of the first tab.");
            });
            TabPanel::show_ctx(ctx, model.tabs_index, 1, |ctx| {
                ctx.ui.label("Content of the second tab.");
                ctx.ui.label("With multiple lines.");
            });
            TabPanel::show_ctx(ctx, model.tabs_index, 2, |ctx| {
                ctx.ui.label("Content of the third tab.");
                Button::primary("Action").on_click(ctx, Msg::ButtonClicked);
            });
        }

        "Menu" => {
            ctx.ui.heading("Menu");
            ctx.ui.label("Vertical tabs / navigation menu (like Tabs but vertical)");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Simple menu (like vertical Tabs)\nMenu::new(&[\"Home\", \"Settings\", \"Profile\"])\n    .show_with(ctx, model.active, Msg::MenuChanged);\n\n// With icons\nIconMenu::new(&[\n    (\"Home\", icons::HOUSE),\n    (\"Settings\", icons::GEAR),\n]).show_with(ctx, model.active, Msg::Changed);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Basic Menu:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                // Left: Menu
                ctx.vertical(|ctx| {
                    Menu::new(&["Overview", "Components", "Settings", "Help"])
                        .show_with(ctx, model.menu_index, Msg::MenuChanged);
                });

                ctx.ui.add_space(16.0);

                // Right: Content
                ctx.vertical(|ctx| {
                    ctx.ui.label(format!("Selected: {}", ["Overview", "Components", "Settings", "Help"][model.menu_index]));
                    ctx.ui.add_space(8.0);
                    match model.menu_index {
                        0 => ctx.ui.label("Welcome to the overview page."),
                        1 => ctx.ui.label("Browse available components."),
                        2 => ctx.ui.label("Configure your preferences."),
                        _ => ctx.ui.label("Get help and support."),
                    };
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("IconMenu:");
            ctx.ui.add_space(8.0);

            IconMenu::new(&[
                ("Home", icons::HOUSE),
                ("Settings", icons::GEAR),
                ("User", icons::USER),
                ("Info", icons::INFO),
            ]).show_with(ctx, model.menu_index, Msg::MenuChanged);
        }

        "Modal" => {
            ctx.ui.heading("Modal");
            ctx.ui.label("Dialog overlays");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Button::primary("Open Modal").on_click(ctx, Msg::OpenModal);
                Button::secondary("Open Confirm").on_click(ctx, Msg::OpenConfirm);
            });

            if let Some(result) = model.confirm_result {
                ctx.ui.add_space(8.0);
                ctx.ui.label(format!("Confirm result: {}", if result { "Yes" } else { "No" }));
            }
        }

        "Table" => {
            ctx.ui.heading("Table");
            ctx.ui.label("Data table components (simple Table + typed DataTable)");
            ctx.ui.add_space(16.0);

            // Simple Table
            ctx.ui.label("Simple Table:");
            let rows: Vec<Vec<String>> = model.table_data
                .iter()
                .map(|(name, age, active)| vec![
                    name.clone(),
                    age.to_string(),
                    if *active { "Yes" } else { "No" }.to_string(),
                ])
                .collect();

            Table::new(&["Name", "Age", "Active"])
                .rows(rows)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            // DataTable with egui_extras
            ctx.ui.label("DataTable (egui_extras - resizable columns):");
            DataTable::new(&model.table_data)
                .column("Name", |(name, _, _)| name.clone())
                .column("Age", |(_, age, _)| age.to_string())
                .column("Status", |(_, _, active)| {
                    if *active { "Active".to_string() } else { "Inactive".to_string() }
                })
                .striped(true)
                .resizable(true)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            // Strip layout demo
            ctx.ui.label("Strip layout (horizontal):");
            Strip::horizontal()
                .exact(80.0)
                .remainder()
                .exact(60.0)
                .show(ctx.ui, |i, ui| {
                    match i {
                        0 => { ui.label("Fixed 80px"); }
                        1 => { ui.label("Remainder (flex)"); }
                        2 => { ui.label("Fixed 60px"); }
                        _ => {}
                    }
                });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"// Simple table
Table::new(&["Col1", "Col2"])
    .rows(vec![vec!["a", "b"]])
    .show(ui);

// Typed DataTable (egui_extras)
DataTable::new(&items)
    .column("Name", |item| item.name.clone())
    .resizable(true)
    .show(ui);

// Strip layout
Strip::horizontal()
    .exact(100.0)
    .remainder()
    .show(ui, |i, ui| { ... });"#).show(ctx.ui);
        }

        "Navbar" => {
            ctx.ui.heading("Navbar");
            ctx.ui.label("Navigation bar (see counter example for full demo)");
            ctx.ui.add_space(8.0);
            ctx.ui.label("The navbar component is used at the top of the counter example.");
        }

        "ErrorConsole" => {
            ctx.ui.heading("ErrorConsole");
            ctx.ui.label("Dismissible error/warning/info message display with 5 severity levels");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Add errors to state\nmodel.errors.push(\"Error message\");\nmodel.errors.push_warning(\"Warning message\");\nmodel.errors.push_info(\"Info message\");\nmodel.errors.push_with_level(\"Debug info\", ErrorLevel::Debug);\nmodel.errors.push_with_level(\"Critical!\", ErrorLevel::Critical);\n\n// Display in view\nErrorConsole::show(ctx, &model.errors, Msg::ErrorConsoleMsg);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Severity Levels:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Button::secondary("Debug").on_click(ctx, Msg::ErrorConsolePush(ErrorLevel::Debug));
                Button::info("Info").on_click(ctx, Msg::ErrorConsolePush(ErrorLevel::Info));
                Button::warning("Warning").on_click(ctx, Msg::ErrorConsolePush(ErrorLevel::Warning));
                Button::danger("Error").on_click(ctx, Msg::ErrorConsolePush(ErrorLevel::Error));
                Button::danger("Critical").on_click(ctx, Msg::ErrorConsolePush(ErrorLevel::Critical));
            });

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Live Demo:");
            ctx.ui.add_space(8.0);

            ErrorConsole::show(ctx, &model.error_console, Msg::ErrorConsoleMsg);

            if model.error_console.is_empty() {
                ctx.ui.label("(Click buttons above to add messages)");
            }
        }

        "Toast" => {
            ctx.ui.heading("Toast Notifications");
            ctx.ui.label("Temporary notifications that auto-dismiss");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Add toast (returns auto-dismiss Cmd)\nmodel.toasts.success(\"Saved!\", Duration::from_secs(3), Msg::DismissToast)\n\n// Handle dismiss\nMsg::DismissToast(id) => model.toasts.dismiss(id)"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Variants:");
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                Button::info("Info").on_click(ctx, Msg::ShowToastInfo);
                Button::success("Success").on_click(ctx, Msg::ShowToastSuccess);
            });
            ctx.ui.add_space(4.0);
            ctx.horizontal(|ctx| {
                Button::warning("Warning").on_click(ctx, Msg::ShowToastWarning);
                Button::danger("Error").on_click(ctx, Msg::ShowToastError);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.label("Click buttons to show toasts (top-right corner)");
            ctx.ui.label("Toasts auto-dismiss after 3-5 seconds");

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Features:");
            ctx.ui.label("- 4 variants: Info, Success, Warning, Error");
            ctx.ui.label("- Auto-dismiss with configurable duration");
            ctx.ui.label("- Manual dismiss via close button");
            ctx.ui.label("- Position: TopRight (default), BottomRight, etc.");
            ctx.ui.label("- Multiple toasts stack vertically");
        }

        "Form" => {
            ctx.ui.heading("Form");
            ctx.ui.label("Combines ValidatedInput atoms into structured forms (TEA-style)");
            ctx.ui.add_space(8.0);

            Code::new(
                "Form::new()\n    .field(\"Email\", &model.email, &state, Msg::EmailChanged)\n    .password_field(\"Password\", &model.pw, &state, Msg::PwChanged)\n    .submit_button(\"Sign Up\")\n    .submit_if(model.can_submit())\n    .on_submit(Msg::Submit)\n    .show(ctx)"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Live Demo:");
            ctx.ui.add_space(8.0);

            // Check if form is valid for submit
            let can_submit = model.email_validation.is_valid() && model.password_validation.is_valid();

            Form::new()
                .field("Email", &model.email_value, &model.email_validation, Msg::EmailChanged)
                .password_field("Password", &model.password_value, &model.password_validation, Msg::PasswordChanged)
                .submit_button("Sign Up")
                .submit_if(can_submit)
                .on_submit(Msg::FormSubmit)
                .show(ctx);

            ctx.ui.add_space(8.0);
            if !can_submit {
                ctx.ui.label("Fill in valid email and password (8+ chars) to enable submit");
            }

            if model.form_submitted {
                ctx.ui.add_space(8.0);
                Badge::success("Form was submitted!").show(ctx.ui);
            }
        }

        "Columns" => {
            ctx.ui.heading("Column Layouts");
            ctx.ui.label("Multi-column layouts with full emit() capability");
            ctx.ui.add_space(8.0);

            Code::new(
                "ctx.two_columns(\n    |ctx| { ctx.button(\"Left\", Msg::Left); },\n    |ctx| { ctx.button(\"Right\", Msg::Right); },\n);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();

            // Two columns demo
            ctx.ui.add_space(8.0);
            ctx.ui.strong("two_columns:");
            ctx.ui.add_space(4.0);

            ctx.two_columns(
                |ctx| {
                    ctx.ui.label("Left Column");
                    Button::primary("Click Left").on_click(ctx, Msg::ColClick(0));
                    ctx.ui.label(format!("Clicks: {}", model.col_clicks[0]));
                },
                |ctx| {
                    ctx.ui.label("Right Column");
                    Button::secondary("Click Right").on_click(ctx, Msg::ColClick(1));
                    ctx.ui.label(format!("Clicks: {}", model.col_clicks[1]));
                },
            );

            ctx.ui.add_space(16.0);
            ctx.ui.separator();

            // Three columns demo
            ctx.ui.add_space(8.0);
            ctx.ui.strong("three_columns:");
            ctx.ui.add_space(4.0);

            ctx.three_columns(
                |ctx| {
                    ctx.ui.label("Col 1");
                    Button::primary("A").on_click(ctx, Msg::ColClick(0));
                },
                |ctx| {
                    ctx.ui.label("Col 2");
                    Button::secondary("B").on_click(ctx, Msg::ColClick(1));
                },
                |ctx| {
                    ctx.ui.label("Col 3");
                    Button::danger("C").on_click(ctx, Msg::ColClick(2));
                },
            );

            ctx.ui.add_space(16.0);
            ctx.ui.separator();

            // Four columns demo
            ctx.ui.add_space(8.0);
            ctx.ui.strong("four_columns:");
            ctx.ui.add_space(4.0);

            ctx.four_columns(
                |ctx| { Button::primary("1").on_click(ctx, Msg::ColClick(0)); },
                |ctx| { Button::secondary("2").on_click(ctx, Msg::ColClick(1)); },
                |ctx| { Button::danger("3").on_click(ctx, Msg::ColClick(2)); },
                |ctx| { Button::ghost("4").on_click(ctx, Msg::ColClick(3)); },
            );

            ctx.ui.add_space(16.0);
            ctx.ui.separator();

            // Variable-length columns demo
            ctx.ui.add_space(8.0);
            ctx.ui.strong("columns (variable-length, 6 cols):");
            ctx.ui.add_space(4.0);

            ctx.columns(vec![
                Box::new(|ctx| { ctx.ui.label("A"); }),
                Box::new(|ctx| { ctx.ui.label("B"); }),
                Box::new(|ctx| { ctx.ui.label("C"); }),
                Box::new(|ctx| { ctx.ui.label("D"); }),
                Box::new(|ctx| { ctx.ui.label("E"); }),
                Box::new(|ctx| { ctx.ui.label("F"); }),
            ]);
        }

        "Conditionals" => {
            ctx.ui.heading("Conditional Rendering");
            ctx.ui.label("show_if, enabled_if, visible_if helpers");
            ctx.ui.add_space(16.0);

            // show_if demo
            ctx.ui.strong("show_if:");
            ctx.ui.add_space(4.0);
            Code::new("ctx.show_if(condition, |ctx| { ... });").show(ctx.ui);
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                ctx.ui.checkbox(&mut model.cond_show.clone(), "Show content");
                if ctx.ui.checkbox(&mut model.cond_show.clone(), "").changed() {
                    ctx.emit(Msg::ToggleCondShow);
                }
            });

            ctx.show_if(model.cond_show, |ctx| {
                ctx.horizontal(|ctx| {
                    Icon::check().color(egui::Color32::GREEN).show(ctx.ui);
                    ctx.ui.label("This content is conditionally shown!");
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();

            // enabled_if demo
            ctx.ui.add_space(8.0);
            ctx.ui.strong("enabled_if:");
            ctx.ui.add_space(4.0);
            Code::new("ctx.enabled_if(can_submit, |ctx| { ... });").show(ctx.ui);
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                if ctx.ui.checkbox(&mut model.cond_enabled.clone(), "Enable button").changed() {
                    ctx.emit(Msg::ToggleCondEnabled);
                }
            });

            ctx.enabled_if(model.cond_enabled, |ctx| {
                Button::primary("Submit").on_click(ctx, Msg::ButtonClicked);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();

            // visible_if demo
            ctx.ui.add_space(8.0);
            ctx.ui.strong("visible_if (keeps space):");
            ctx.ui.add_space(4.0);
            Code::new("ctx.visible_if(show_hint, |ctx| { ... });").show(ctx.ui);
            ctx.ui.add_space(8.0);

            ctx.horizontal(|ctx| {
                if ctx.ui.checkbox(&mut model.cond_visible.clone(), "Show hint").changed() {
                    ctx.emit(Msg::ToggleCondVisible);
                }
            });

            ctx.visible_if(model.cond_visible, |ctx| {
                ctx.ui.label("This hint takes space even when hidden");
            });
            ctx.horizontal(|ctx| {
                Icon::arrow_left().show_ctx(ctx);
                ctx.ui.label("Content below stays in place");
            });
        }

        "Dock" => {
            ctx.ui.heading("Dock");
            ctx.ui.label("Dockable panel layout with tabs (wraps egui_dock)");
            ctx.ui.add_space(8.0);

            Code::new(r#"// Create dock layout
let dock = dock_layout::three_column(
    Pane::Browser,
    Pane::Editor,
    Pane::Inspector,
    0.2, 0.2,
);

// Show dock area
DockArea::new(&mut model.dock)
    .show_close_buttons(true)
    .show_add_buttons(true)
    .show(ui, |ui, pane| {
        match pane {
            Pane::Browser => ui.label("Browser"),
            Pane::Editor => ui.label("Editor"),
            _ => ui.label("Other"),
        }
    });"#).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Interactive Demo:");
            ctx.ui.label("Try dragging tabs, closing them, or clicking + to add new tabs.");
            ctx.ui.add_space(8.0);

            // Use fixed-size area for dock
            let available = ctx.ui.available_size();
            let dock_height = 350.0_f32.min(available.y - 50.0).max(200.0);

            egui::Frame::default()
                .stroke(egui::Stroke::new(1.0, ctx.ui.visuals().widgets.noninteractive.bg_stroke.color))
                .inner_margin(4.0)
                .show(ctx.ui, |ui| {
                    ui.set_min_size(egui::vec2(available.x - 20.0, dock_height));

                    DockArea::new(&mut model.dock.borrow_mut())
                        .show_close_buttons(true)
                        .show_add_buttons(true)
                        .tabs_are_draggable(true)
                        .show(ui, |ui, pane| {
                            match pane {
                                DemoPane::Browser => {
                                    ui.heading("Browser");
                                    ui.label("File browser pane");
                                    ui.separator();
                                    ui.label("📁 src/");
                                    ui.label("  📄 main.rs");
                                    ui.label("  📄 lib.rs");
                                    ui.label("📁 tests/");
                                }
                                DemoPane::Editor => {
                                    ui.heading("Editor");
                                    ui.label("Code editor pane");
                                    ui.separator();
                                    ui.code("fn main() {\n    println!(\"Hello!\");\n}");
                                }
                                DemoPane::Console => {
                                    ui.heading("Console");
                                    ui.label("Terminal/console pane");
                                    ui.separator();
                                    ui.label("> cargo run");
                                    ui.label("   Compiling...");
                                    ui.label("   Finished");
                                }
                                DemoPane::Inspector => {
                                    ui.heading("Inspector");
                                    ui.label("Property inspector pane");
                                    ui.separator();
                                    ui.label("Name: main.rs");
                                    ui.label("Size: 256 bytes");
                                    ui.label("Modified: Today");
                                }
                            }
                        });
                });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Layout Helpers:");
            Code::new(r#"// Preset layouts
dock_layout::left_right(left, right, 0.3);
dock_layout::top_bottom(top, bottom, 0.7);
dock_layout::three_column(l, c, r, 0.2, 0.2);
dock_layout::daw(browser, main, inspector, timeline);
dock_layout::vscode(sidebar, editors, terminals);"#).show(ctx.ui);
        }

        #[cfg(feature = "snarl")]
        "NodeGraph" => {
            ctx.ui.heading("NodeGraph");
            ctx.ui.label("Visual node-based editor (wraps egui-snarl)");
            ctx.ui.add_space(8.0);

            Code::new(r#"// Define node type
enum MyNode {
    Source { name: String },
    Effect { intensity: f32 },
    Output,
}

// Implement SnarlViewer for your node type
impl SnarlViewer<MyNode> for MyViewer {
    fn title(&mut self, node: &MyNode) -> String { ... }
    fn inputs(&mut self, node: &MyNode) -> usize { ... }
    fn outputs(&mut self, node: &MyNode) -> usize { ... }
    fn show_input(...) -> PinInfo { ... }
    fn show_output(...) -> PinInfo { ... }
}

// Show the node graph
NodeGraphArea::new(&mut model.graph)
    .graph_action(MenuAction::new("add", "Add Node"))
    .show(ui, &mut MyViewer);"#).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Interactive Demo:");
            ctx.ui.label("Drag nodes to move, connect pins by dragging, right-click for context menu.");
            ctx.ui.add_space(8.0);

            // Show last event
            if let Some(ref event) = model.node_graph_last_event {
                ctx.ui.label(format!("Last event: {}", event));
            }
            ctx.ui.add_space(8.0);

            // Use fixed-size area for node graph
            let available = ctx.ui.available_size();
            let graph_height = 350.0_f32.min(available.y - 50.0).max(200.0);

            egui::Frame::default()
                .stroke(egui::Stroke::new(1.0, ctx.ui.visuals().widgets.noninteractive.bg_stroke.color))
                .inner_margin(4.0)
                .show(ctx.ui, |ui| {
                    ui.set_min_size(egui::vec2(available.x - 20.0, graph_height));

                    NodeGraphArea::new(&mut model.node_graph.borrow_mut())
                        .show(ui, &mut DemoNodeViewer);
                });

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Features:");
            ctx.ui.label("• Theme-aware styling via NodeGraphStyle");
            ctx.ui.label("• Custom context menu actions via MenuAction");
            ctx.ui.label("• TEA-style events via NodeGraphEvent");
            ctx.ui.label("• Preset pin types for VJ/DAW (Audio, Video, MIDI, etc.)");
        }

        #[cfg(not(feature = "snarl"))]
        "NodeGraph" => {
            ctx.ui.heading("NodeGraph");
            ctx.ui.label("Node graph editor (requires 'snarl' feature)");
            ctx.ui.add_space(8.0);
            Code::new("cargo run --example storybook --features snarl").show(ctx.ui);
        }

        "NodeLayout" => {
            ctx.ui.heading("NodeLayout");
            ctx.ui.label("Infinite canvas pane layout with pan/zoom");
            ctx.ui.add_space(8.0);

            Code::new(r#"// Create layout with panes
let mut layout = NodeLayout::new();
layout.add_pane(
    LayoutPane::new("preview", "Preview")
        .with_size(300.0, 200.0),
    pos2(20.0, 20.0),
);

// Show the layout
NodeLayoutArea::new(&mut layout, |ui, pane| {
    match pane.id.as_str() {
        "preview" => ui.label("Preview content"),
        _ => {}
    }
}).show(ui);"#).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Interactive Demo:");
            ctx.ui.label("Drag panes to move them on the infinite canvas. Pan/zoom supported.");
            ctx.ui.add_space(8.0);

            // Menu bar toggle
            ctx.horizontal(|ctx| {
                let menu_label = if model.node_layout_show_menu_bar { "☑ Show Menu Bar" } else { "☐ Show Menu Bar" };
                Button::new(menu_label).on_click(ctx, Msg::ToggleNodeLayoutMenuBar);
                ctx.ui.label("(Lock and Arrange controls are in the menu bar)");
            });
            ctx.ui.add_space(8.0);

            // Use fixed-size area for node layout
            let available = ctx.ui.available_size();
            let layout_height = 350.0_f32.min(available.y - 50.0).max(200.0);

            // Capture lock change events from menu bar
            let lock_event: std::cell::Cell<Option<egui_cha_ds::LockLevel>> = std::cell::Cell::new(None);

            egui::Frame::default()
                .stroke(egui::Stroke::new(1.0, ctx.ui.visuals().widgets.noninteractive.bg_stroke.color))
                .inner_margin(4.0)
                .show(ctx.ui, |ui| {
                    ui.set_min_size(egui::vec2(available.x - 20.0, layout_height));

                    let lock_level = model.node_layout_lock_level;
                    let mut binding = model.node_layout.borrow_mut();
                    let events = NodeLayoutArea::new(&mut binding, |ui, pane| {
                        // Render pane content based on id
                        match pane.id.as_str() {
                            "preview" => {
                                ui.label("Preview Area");
                                ui.add_space(4.0);
                                // Placeholder for video
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(260.0, 120.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    4.0,
                                    egui::Color32::from_gray(40),
                                );
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "Video Output",
                                    egui::FontId::default(),
                                    egui::Color32::GRAY,
                                );
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    if ui.button("⏮").clicked() {}
                                    if ui.button("▶").clicked() {}
                                    if ui.button("⏭").clicked() {}
                                });
                            }
                            "effects" => {
                                ui.label("Effects Chain");
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut true, "Blur");
                                    ui.add(egui::Slider::new(&mut 0.5_f32, 0.0..=1.0).text(""));
                                });
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut true, "Glow");
                                    ui.add(egui::Slider::new(&mut 0.3_f32, 0.0..=1.0).text(""));
                                });
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut false, "Distort");
                                    ui.add(egui::Slider::new(&mut 0.0_f32, 0.0..=1.0).text(""));
                                });
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    if ui.button("+ Add").clicked() {}
                                    if ui.button("Reset").clicked() {}
                                });
                            }
                            "layers" => {
                                ui.label("Layer Stack");
                                ui.add_space(4.0);
                                for i in 1..=3 {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut true, "");
                                        ui.label(format!("Layer {}", i));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.small_button("🗑");
                                            ui.small_button("👁");
                                        });
                                    });
                                }
                                ui.add_space(4.0);
                                if ui.button("+ New Layer").clicked() {}
                            }
                            _ => {
                                ui.label(&pane.title);
                            }
                        }
                    })
                    .lock_level(lock_level)
                    .show_menu_bar(model.node_layout_show_menu_bar)
                    .show(ui);

                    // Capture lock change events for processing outside closure
                    for event in events {
                        if let egui_cha_ds::NodeLayoutEvent::CanvasLockChanged(new_level) = event {
                            lock_event.set(Some(new_level));
                        }
                    }
                });

            // Handle captured lock event via Elm architecture
            if let Some(new_level) = lock_event.get() {
                ctx.emit(Msg::SetNodeLayoutLock(new_level));
            }

            ctx.ui.add_space(16.0);

            ctx.ui.strong("Features:");
            ctx.ui.label("• Infinite canvas with pan/zoom (via egui Scene)");
            ctx.ui.label("• Free-form pane positioning");
            ctx.ui.label("• Lock mode to prevent changes");
            ctx.ui.label("• Custom content via closure");
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

fn render_framework(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match FRAMEWORK[model.active_component] {
        "Cmd::delay" => {
            ctx.ui.heading("Cmd::delay");
            ctx.ui.label("Delayed message emission");
            ctx.ui.add_space(8.0);

            Code::new("Cmd::delay(Duration::from_secs(2), Msg::Complete)").show(ctx.ui);

            ctx.ui.add_space(12.0);

            if model.delay_pending {
                ctx.ui.label("Waiting... (2 seconds)");
                ctx.ui.spinner();
            } else {
                Button::primary("Start 2s Delay").on_click(ctx, Msg::StartDelay);
            }

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Completed: {} times", model.delay_count));
        }

        "Cmd::timeout" => {
            ctx.ui.heading("Cmd::with_timeout");
            ctx.ui.label("Async task with timeout");
            ctx.ui.add_space(8.0);

            Code::new(
                "Cmd::with_timeout(\n    Duration::from_secs(1),\n    async_task(),\n    |result| Msg::Success,\n    Msg::Timeout,\n)"
            ).show(ctx.ui);

            ctx.ui.add_space(12.0);

            Button::primary("Start Task (500ms, timeout 1s)").on_click(ctx, Msg::StartTimeout);

            ctx.ui.add_space(8.0);
            if let Some(status) = model.timeout_status {
                Badge::info(status).show(ctx.ui);
            }
        }

        "Cmd::retry" => {
            ctx.ui.heading("Cmd::retry");
            ctx.ui.label("Exponential backoff retry");
            ctx.ui.add_space(8.0);

            Code::new(
                "Cmd::retry(\n    3, // max attempts\n    Duration::from_millis(500),\n    || async_task(),\n    |ok| Msg::Success,\n    |err, attempts| Msg::Failed,\n)"
            ).show(ctx.ui);

            ctx.ui.add_space(12.0);

            Button::primary("Start Retry (fails twice, succeeds on 3rd)")
                .on_click(ctx, Msg::StartRetry);

            ctx.ui.add_space(8.0);
            if let Some(status) = &model.retry_status {
                ctx.ui.label(status);
            }
        }

        "Sub::interval" => {
            ctx.ui.heading("Sub::interval");
            ctx.ui.label("Periodic timer subscription");
            ctx.ui.add_space(8.0);

            Code::new(
                "fn subscriptions(model: &Model) -> Sub<Msg> {\n    if model.enabled {\n        Sub::interval(\"timer\", Duration::from_secs(1), Msg::Tick)\n    } else {\n        Sub::none()\n    }\n}"
            ).show(ctx.ui);

            ctx.ui.add_space(12.0);

            let label = if model.interval_enabled { "Stop Interval" } else { "Start Interval (1s)" };
            let variant = if model.interval_enabled { Button::danger(label) } else { Button::primary(label) };
            variant.on_click(ctx, Msg::ToggleInterval);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Tick count: {}", model.interval_count));

            if model.interval_enabled {
                Badge::success("Running").show(ctx.ui);
            } else {
                Badge::new("Stopped").show(ctx.ui);
            }
        }

        "Debouncer" => {
            ctx.ui.heading("Debouncer");
            ctx.ui.label("Delays action until input stops (500ms)");
            ctx.ui.add_space(8.0);

            Code::new(
                "// In update:\nMsg::Input(text) => {\n    model.debouncer.trigger(Duration::from_millis(500), Msg::Search)\n}\nMsg::Search => {\n    if model.debouncer.should_fire() {\n        // Execute search\n    }\n}"
            ).show(ctx.ui);

            ctx.ui.add_space(12.0);

            ctx.ui.label("Type quickly - search only fires after you stop:");
            Input::new()
                .placeholder("Type to search...")
                .show_with(ctx, &model.debounce_input, Msg::DebounceInput);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Search executed: {} times", model.debounce_search_count));
            ctx.ui.label("(Try typing fast - notice search doesn't fire on every keystroke)");
        }

        "Throttler" => {
            ctx.ui.heading("Throttler");
            ctx.ui.label("Limits action frequency (500ms)");
            ctx.ui.add_space(8.0);

            Code::new(
                "// In update:\nMsg::Click => {\n    model.throttler.run(Duration::from_millis(500), || {\n        Cmd::msg(Msg::ActualAction)\n    })\n}"
            ).show(ctx.ui);

            ctx.ui.add_space(12.0);

            ctx.ui.label("Click rapidly - action only fires once per 500ms:");
            Button::primary("Click me rapidly!").on_click(ctx, Msg::ThrottleClick);

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Button clicks: {}", model.throttle_click_count));
            ctx.ui.label(format!("Actual actions: {}", model.throttle_actual_count));
            ctx.ui.label("(Notice actual actions are throttled)");
        }

        "Drag & Drop" => {
            ctx.ui.heading("Drag & Drop");
            ctx.ui.label("Type-safe drag and drop with TEA messages");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Drag source\nctx.drag_source(\"item\", payload, |ctx| {\n    ctx.ui.label(\"Drag me\");\n});\n\n// Drop zone\nctx.drop_zone::<Payload, _>(|ctx| {\n    ctx.ui.label(\"Drop here\");\n}).on_drop(ctx, |p| Msg::Dropped(p));"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Live Demo:");
            ctx.ui.label("Drag items from left to right");
            ctx.ui.add_space(8.0);

            ctx.two_columns(
                |ctx| {
                    ctx.ui.label("Available Items:");
                    ctx.ui.add_space(4.0);

                    for item in &model.dnd_items {
                        ctx.drag_source(
                            egui::Id::new(item),
                            item.clone(),
                            |ctx| {
                                ctx.ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        Icon::hash().size(16.0).show(ui);
                                        ui.label(item);
                                    });
                                });
                            },
                        );
                    }

                    if model.dnd_items.is_empty() {
                        ctx.ui.label("(empty)");
                    }
                },
                |ctx| {
                    ctx.ui.label("Drop Zone:");
                    ctx.ui.add_space(4.0);

                    let drop_resp = ctx.drop_zone::<String, _>(|ctx| {
                        ctx.ui.group(|ui| {
                            ui.set_min_size(egui::vec2(150.0, 100.0));
                            if model.dnd_dropped.is_empty() {
                                ui.label("Drop items here...");
                            } else {
                                for item in &model.dnd_dropped {
                                    ui.horizontal(|ui| {
                                        Icon::check().size(16.0).show(ui);
                                        ui.label(item);
                                    });
                                }
                            }
                        });
                    });

                    drop_resp.on_drop(ctx, |item| Msg::DndDropped((*item).clone()));
                },
            );
        }

        "Shortcuts" => {
            ctx.ui.heading("Keyboard Shortcuts");
            ctx.ui.label("TEA-style keyboard input handling");
            ctx.ui.add_space(8.0);

            Code::new(
                "use egui_cha::prelude::*;\n\n// Use standard shortcuts from the shortcuts module\nctx.on_shortcut(shortcuts::SAVE, Msg::Save);\nctx.on_shortcut(shortcuts::UNDO, Msg::Undo);\n\n// Use on_key for simple key presses\nctx.on_key(Key::Escape, Msg::Cancel);\n\n// Define custom shortcuts\nconst MY_SHORTCUT: KeyboardShortcut = \n    KeyboardShortcut::new(Modifiers::COMMAND, Key::T);\nctx.on_shortcut(MY_SHORTCUT, Msg::Custom);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Live Demo:");
            ctx.ui.label("Try these keyboard shortcuts:");
            ctx.ui.add_space(8.0);

            // Register shortcuts for this demo
            ctx.on_shortcut(shortcuts::SAVE, Msg::ShortcutSave);
            ctx.on_shortcut(shortcuts::UNDO, Msg::ShortcutUndo);
            ctx.on_key(Key::Escape, Msg::ShortcutReset);

            // Arrow keys for counter
            ctx.on_key(Key::ArrowUp, Msg::ShortcutIncrement);
            ctx.on_key(Key::ArrowDown, Msg::ShortcutDecrement);

            // Custom shortcuts
            const INCREMENT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Plus);
            const DECREMENT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Minus);
            ctx.on_shortcut(INCREMENT, Msg::ShortcutIncrement);
            ctx.on_shortcut(DECREMENT, Msg::ShortcutDecrement);

            ctx.ui.label("Cmd+S : Save action");
            ctx.ui.label("Cmd+Z : Undo action");
            ctx.ui.label("Escape : Reset counter");
            ctx.ui.label("Arrow Up/Down or +/- : Counter");

            ctx.ui.add_space(16.0);

            ctx.horizontal(|ctx| {
                ctx.ui.strong("Counter:");
                ctx.ui.label(format!("{}", model.shortcut_counter));
            });

            if let Some(action) = model.shortcut_last_action {
                ctx.ui.add_space(8.0);
                Badge::success(action).show(ctx.ui);
            }

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Available Constants in shortcuts module:");
            ctx.ui.label("Pre-defined KeyboardShortcut constants. Use as shortcuts::SAVE, shortcuts::UNDO, etc.");
            ctx.ui.add_space(4.0);

            ctx.two_columns(
                |ctx| {
                    ctx.ui.label("File: NEW, OPEN, SAVE, CLOSE");
                    ctx.ui.label("Edit: UNDO, REDO, CUT, COPY, PASTE");
                },
                |ctx| {
                    ctx.ui.label("Search: FIND, REPLACE");
                    ctx.ui.label("Common: ESCAPE, ENTER, TAB");
                },
            );
        }

        "Dynamic Bindings" => {
            ctx.ui.heading("Dynamic Key Bindings");
            ctx.ui.label("Runtime-rebindable keyboard shortcuts (Phase 2)");
            ctx.ui.add_space(8.0);

            Code::new(
                "use egui_cha::prelude::*;\n\n#[derive(Clone, PartialEq, Eq, Hash)]\nenum Action { Save, Undo, Redo }\n\n// Create bindings with defaults\nlet bindings = ActionBindings::new()\n    .with_default(Action::Save, shortcuts::SAVE)\n    .with_default(Action::Undo, shortcuts::UNDO);\n\n// Rebind at runtime\nbindings.rebind(&Action::Save, DynamicShortcut::new(\n    Modifiers::CTRL | Modifiers::SHIFT, Key::S\n));\n\n// In view function\nctx.on_action(&bindings, &Action::Save, Msg::Save);"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Live Demo:");
            ctx.ui.label("Try the shortcuts, then rebind them!");
            ctx.ui.add_space(8.0);

            // Register action bindings
            ctx.on_action(&model.bindings, &DemoAction::Increment, Msg::BindingsAction(DemoAction::Increment));
            ctx.on_action(&model.bindings, &DemoAction::Decrement, Msg::BindingsAction(DemoAction::Decrement));
            ctx.on_action(&model.bindings, &DemoAction::Reset, Msg::BindingsAction(DemoAction::Reset));
            ctx.on_action(&model.bindings, &DemoAction::Save, Msg::BindingsAction(DemoAction::Save));

            // Current bindings table
            ctx.ui.strong("Current Bindings:");
            ctx.ui.label("Press the shortcut key to trigger the action. Use buttons below to rebind.");
            ctx.ui.add_space(4.0);

            egui::Grid::new("bindings_grid")
                .num_columns(3)
                .spacing([20.0, 4.0])
                .show(ctx.ui, |ui| {
                    ui.strong("Action");
                    ui.strong("Shortcut");
                    ui.strong("Modified");
                    ui.end_row();

                    for action in [DemoAction::Increment, DemoAction::Decrement, DemoAction::Reset, DemoAction::Save] {
                        let label = match &action {
                            DemoAction::Increment => "Increment (counter +1)",
                            DemoAction::Decrement => "Decrement (counter -1)",
                            DemoAction::Reset => "Reset (counter = 0)",
                            DemoAction::Save => "Save (show badge)",
                        };

                        ui.label(label);

                        if let Some(shortcut) = model.bindings.get(&action) {
                            ui.label(shortcut.display());
                        } else {
                            ui.label("-");
                        }

                        if model.bindings.is_modified(&action) {
                            Badge::warning("Modified").show(ui);
                        } else {
                            ui.label("-");
                        }

                        ui.end_row();
                    }
                });

            ctx.ui.add_space(16.0);

            // Quick rebind buttons
            ctx.ui.strong("Quick Rebind (try these):");
            ctx.ui.add_space(4.0);

            ctx.horizontal(|ctx| {
                if ctx.ui.button("Increment -> W").clicked() {
                    ctx.emit(Msg::BindingsRebind(
                        DemoAction::Increment,
                        DynamicShortcut::new(Modifiers::NONE, Key::W),
                    ));
                }
                if ctx.ui.button("Decrement -> S").clicked() {
                    ctx.emit(Msg::BindingsRebind(
                        DemoAction::Decrement,
                        DynamicShortcut::new(Modifiers::NONE, Key::S),
                    ));
                }
            });

            ctx.horizontal(|ctx| {
                if ctx.ui.button("Save -> Ctrl+Shift+S").clicked() {
                    ctx.emit(Msg::BindingsRebind(
                        DemoAction::Save,
                        DynamicShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::S),
                    ));
                }
                Button::secondary("Reset All to Defaults").on_click(ctx, Msg::BindingsResetAll);
            });

            ctx.ui.add_space(16.0);

            // Counter display
            ctx.horizontal(|ctx| {
                ctx.ui.strong("Counter:");
                ctx.ui.label(format!("{}", model.bindings_counter));
            });

            if let Some(action) = model.bindings_last_action {
                ctx.ui.add_space(8.0);
                Badge::success(action).show(ctx.ui);
            }
        }

        "ScrollArea" => {
            ctx.ui.heading("ScrollArea");
            ctx.ui.label("Scrollable container with configurable options (Core)");
            ctx.ui.add_space(8.0);

            Code::new(
                "// Builder pattern (recommended)\nScrollArea::vertical()\n    .max_height(200.0)\n    .show_ctx(ctx, |ctx| { ... });\n\n// Closure-based\nctx.scroll_area_with(\n    |area| area.max_height(200.0),\n    |ctx| { ... },\n);\n\n// Simple shortcuts\nctx.scroll_area(|ctx| { ... });\nctx.scroll_area_id(\"id\", |ctx| { ... });"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("ScrollArea::vertical() - Builder pattern:");
            ctx.ui.add_space(4.0);

            ScrollArea::vertical()
                .id_salt("framework_demo_1")
                .max_height(120.0)
                .show_ctx(ctx, |ctx| {
                    for i in 0..20 {
                        ctx.ui.label(format!("Builder item {}", i));
                    }
                });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("ScrollArea::horizontal():");
            ctx.ui.add_space(4.0);

            ScrollArea::horizontal()
                .id_salt("framework_demo_2")
                .show_ctx(ctx, |ctx| {
                    ctx.horizontal(|ctx| {
                        for i in 0..15 {
                            Button::outline(&format!("Btn {}", i)).show(ctx.ui);
                        }
                    });
                });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("ctx.scroll_area_with() - Closure-based:");
            ctx.ui.add_space(4.0);

            ctx.scroll_area_with(
                |area| area.id_salt("framework_demo_3").max_height(100.0).auto_shrink([false, false]),
                |ctx| {
                    for i in 0..15 {
                        ctx.ui.label(format!("Closure item {} (no shrink)", i));
                    }
                },
            );

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Options:");
            ctx.ui.label("- .max_height() / .max_width()");
            ctx.ui.label("- .auto_shrink([h, v]) / .no_shrink()");
            ctx.ui.label("- .always_show_scroll() / .hide_scroll()");
            ctx.ui.label("- .animated(bool)");
            ctx.ui.label("- .id_salt(id) for multiple scroll areas");
        }

        "RepaintMode" => {
            ctx.ui.heading("RepaintMode");
            ctx.ui.label("Control frame rate and repaint behavior");
            ctx.ui.add_space(8.0);

            // Get frame stats before borrowing ui
            let egui_ctx = ctx.ui.ctx().clone();
            let frame_time = egui_ctx.input(|i| i.stable_dt);
            let fps = if frame_time > 0.0 { 1.0 / frame_time } else { 0.0 };
            let t = egui_ctx.input(|i| i.time);
            let progress = ((t * 2.0).sin() * 0.5 + 0.5) as f32;

            Card::new().show(ctx.ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Live Stats:");
                    Badge::info(&format!("FPS: {:.1}", fps)).show(ui);
                    ui.separator();
                    ui.label(format!("Frame time: {:.1}ms", frame_time * 1000.0));
                });

                ui.add_space(8.0);

                // Animated progress bar
                ui.horizontal(|ui| {
                    ui.label("Animation:");
                    ui.add(egui::ProgressBar::new(progress).desired_width(200.0));
                });

                ui.add_space(4.0);
                ui.label("(Move mouse or interact to see Reactive mode in action)");
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            Code::new(
                "pub enum RepaintMode {\n    Reactive,      // Event-driven (default)\n    FixedFps(u32), // Fixed frame rate\n    VSync,         // Monitor refresh rate\n}"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Usage:");
            ctx.ui.add_space(4.0);

            Code::new(
                "// Default: Reactive (power-efficient)\negui_cha::run::<MyApp>(RunConfig::new(\"App\"))\n\n// Fixed 60 FPS (animations, VJ software)\negui_cha::run::<MyApp>(\n    RunConfig::new(\"VJ App\")\n        .with_repaint_mode(RepaintMode::FixedFps(60))\n)\n\n// VSync (smooth, monitor-synced)\negui_cha::run::<MyApp>(\n    RunConfig::new(\"Game\")\n        .with_repaint_mode(RepaintMode::VSync)\n)"
            ).show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Mode Comparison:");
            ctx.ui.add_space(4.0);

            egui::Grid::new("repaint_mode_grid")
                .num_columns(3)
                .spacing([20.0, 4.0])
                .show(ctx.ui, |ui| {
                    ui.strong("Mode");
                    ui.strong("Use Case");
                    ui.strong("Power");
                    ui.end_row();

                    ui.label("Reactive");
                    ui.label("Standard UI apps, forms");
                    ui.label("Low (best)");
                    ui.end_row();

                    ui.label("FixedFps(30)");
                    ui.label("Light animations");
                    ui.label("Medium");
                    ui.end_row();

                    ui.label("FixedFps(60)");
                    ui.label("VJ software, real-time viz");
                    ui.label("Medium-High");
                    ui.end_row();

                    ui.label("VSync");
                    ui.label("Games, smooth animations");
                    ui.label("High");
                    ui.end_row();
                });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Notes:");
            ctx.ui.label("• Reactive: Only repaints on user input or pending messages");
            ctx.ui.label("• FixedFps: Uses request_repaint_after() for consistent timing");
            ctx.ui.label("• VSync: Repaints every frame at monitor refresh rate");
            ctx.ui.label("• This storybook uses Reactive mode (default)");
        }

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}

/// Render Theme settings panel
fn render_theme(model: &Model, ctx: &mut ViewCtx<'_, Msg>) {
    match THEME_ITEMS[model.active_component] {
        "Scale Controls" => {
            ctx.ui.heading("Scale Controls");
            ctx.ui.label("Adjust spacing, radius, font, and stroke scales");
            ctx.ui.add_space(8.0);

            // Current theme display
            let theme_name = match model.theme_index {
                0 => "Light",
                1 => "Dark",
                2 => "Pastel",
                _ => "Pastel Dark",
            };
            ctx.ui.horizontal(|ui| {
                ui.strong("Base Theme:");
                Badge::info(theme_name).show(ui);
            });
            ctx.ui.add_space(16.0);

            // Sliders for scales
            let mut spacing = model.spacing_scale;
            if ctx.ui.add(egui::Slider::new(&mut spacing, 0.5..=2.0).text("Spacing Scale")).changed() {
                ctx.emit(Msg::SetSpacingScale(spacing));
            }

            let mut radius = model.radius_scale;
            if ctx.ui.add(egui::Slider::new(&mut radius, 0.0..=3.0).text("Radius Scale")).changed() {
                ctx.emit(Msg::SetRadiusScale(radius));
            }

            let mut font = model.font_scale;
            if ctx.ui.add(egui::Slider::new(&mut font, 0.5..=2.0).text("Font Scale")).changed() {
                ctx.emit(Msg::SetFontScale(font));
            }

            let mut stroke = model.stroke_scale;
            if ctx.ui.add(egui::Slider::new(&mut stroke, 0.5..=3.0).text("Stroke Scale")).changed() {
                ctx.emit(Msg::SetStrokeScale(stroke));
            }

            ctx.ui.add_space(16.0);
            Button::outline("Reset All").on_click(ctx, Msg::ResetThemeScales);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Current Values:");
            let theme = &model.theme;
            egui::Grid::new("scale_values")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ctx.ui, |ui| {
                    ui.label("spacing_sm:");
                    ui.label(format!("{:.1}", theme.spacing_sm));
                    ui.end_row();

                    ui.label("spacing_md:");
                    ui.label(format!("{:.1}", theme.spacing_md));
                    ui.end_row();

                    ui.label("radius_md:");
                    ui.label(format!("{:.1}", theme.radius_md));
                    ui.end_row();

                    ui.label("font_size_md:");
                    ui.label(format!("{:.1}", theme.font_size_md));
                    ui.end_row();

                    ui.label("border_width:");
                    ui.label(format!("{:.1}", theme.border_width));
                    ui.end_row();
                });
        }

        "Shadow & Overlay" => {
            ctx.ui.heading("Shadow & Overlay");
            ctx.ui.label("Configure shadow and modal overlay settings");
            ctx.ui.add_space(16.0);

            // Shadow toggle
            ctx.ui.horizontal(|ui| {
                ui.strong("Shadow:");
                if model.shadow_enabled {
                    Badge::success("Enabled").show(ui);
                } else {
                    Badge::new("Disabled").show(ui);
                }
            });

            let shadow_label = if model.shadow_enabled { "Disable Shadow" } else { "Enable Shadow" };
            Button::outline(shadow_label).on_click(ctx, Msg::ToggleShadow);

            if model.shadow_enabled {
                ctx.ui.add_space(8.0);
                let mut blur = model.shadow_blur;
                if ctx.ui.add(egui::Slider::new(&mut blur, 1.0..=16.0).text("Shadow Blur")).changed() {
                    ctx.emit(Msg::SetShadowBlur(blur));
                }
            }

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(16.0);

            // Overlay dim
            ctx.ui.strong("Modal Overlay:");
            let mut dim = model.overlay_dim;
            if ctx.ui.add(egui::Slider::new(&mut dim, 0.0..=1.0).text("Overlay Dim")).changed() {
                ctx.emit(Msg::SetOverlayDim(dim));
            }

            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Current: {:.0}% black overlay", model.overlay_dim * 100.0));

            ctx.ui.add_space(16.0);
            Button::primary("Test Modal").on_click(ctx, Msg::OpenModal);
        }

        "Preview" => {
            ctx.ui.heading("Theme Preview");
            ctx.ui.label("Live preview of current theme settings");
            ctx.ui.add_space(16.0);

            // Sample components
            ctx.ui.strong("Buttons:");
            ctx.ui.horizontal(|ui| {
                Button::primary("Primary").show(ui);
                Button::secondary("Secondary").show(ui);
                Button::outline("Outline").show(ui);
                Button::ghost("Ghost").show(ui);
            });

            ctx.ui.add_space(8.0);
            ctx.ui.horizontal(|ui| {
                Button::success("Success").show(ui);
                Button::warning("Warning").show(ui);
                Button::danger("Danger").show(ui);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Badges:");
            ctx.ui.horizontal(|ui| {
                Badge::new("Default").show(ui);
                Badge::success("Success").show(ui);
                Badge::warning("Warning").show(ui);
                Badge::error("Error").show(ui);
                Badge::info("Info").show(ui);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Card:");
            Card::titled("Sample Card").show(ctx.ui, |ui| {
                ui.label("This is a card with the current theme applied.");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    Button::primary("Action").show(ui);
                    Button::outline("Cancel").show(ui);
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Input:");
            let mut sample = String::from("Sample text");
            ctx.ui.text_edit_singleline(&mut sample);

            ctx.ui.add_space(16.0);
            ctx.ui.strong("Log Severity Colors:");
            let theme = &model.theme;
            ctx.horizontal(|ctx| {
                Text::small("DEBUG").color(theme.log_debug).show(ctx.ui);
                Text::small("INFO").color(theme.log_info).show(ctx.ui);
                Text::small("WARN").color(theme.log_warn).show(ctx.ui);
                Text::small("ERROR").color(theme.log_error).show(ctx.ui);
                Text::small("CRITICAL").color(theme.log_critical).show(ctx.ui);
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.strong("Theme Summary:");
            egui::Grid::new("theme_summary")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ctx.ui, |ui| {
                    ui.label("Spacing Scale:");
                    ui.label(format!("{:.2}x", model.spacing_scale));
                    ui.end_row();

                    ui.label("Radius Scale:");
                    ui.label(format!("{:.2}x", model.radius_scale));
                    ui.end_row();

                    ui.label("Font Scale:");
                    ui.label(format!("{:.2}x", model.font_scale));
                    ui.end_row();

                    ui.label("Stroke Scale:");
                    ui.label(format!("{:.2}x", model.stroke_scale));
                    ui.end_row();

                    ui.label("Shadow:");
                    ui.label(if model.shadow_enabled {
                        format!("blur: {:.0}", model.shadow_blur)
                    } else {
                        "Off".to_string()
                    });
                    ui.end_row();

                    ui.label("Overlay Dim:");
                    ui.label(format!("{:.0}%", model.overlay_dim * 100.0));
                    ui.end_row();
                });
        }

        _ => {
            ctx.ui.label("Theme item not implemented");
        }
    }
}
