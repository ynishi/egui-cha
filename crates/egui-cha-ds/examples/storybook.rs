//! DS Storybook - Component showcase
//!
//! Visual catalog of all DS components and framework features

use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;
use egui_cha_ds::{ConfirmResult, ToastContainer, ToastId};
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
}

/// Demo action for dynamic bindings showcase
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum DemoAction {
    Increment,
    Decrement,
    Reset,
    Save,
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
}

const CATEGORIES: &[&str] = &["Atoms", "Semantics", "Molecules", "Framework", "Theme"];

const ATOMS: &[&str] = &[
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
    "Waveform",
    "Spectrum",
    "LevelMeter",
    "ArcSlider",
    "Oscilloscope",
    "ButtonGroup",
    "BpmDisplay",
    "ClipGrid",
    "Plot",
    "Link",
    "Code",
    "Text",
    "Tooltip",
    "Context Menu",
    "ListItem",
];

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

                // Component list
                let components = match model.active_category {
                    0 => ATOMS,
                    1 => SEMANTICS,
                    2 => MOLECULES,
                    3 => FRAMEWORK,
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
                        0 => render_atom(model, ctx),
                        1 => render_semantics(model, ctx),
                        2 => render_molecule(model, ctx),
                        3 => render_framework(model, ctx),
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

fn render_atom(model: &Model, ctx: &mut ViewCtx<Msg>) {
    match ATOMS[model.active_component] {
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

        "Waveform" => {
            ctx.ui.heading("Waveform");
            ctx.ui.label("Audio waveform visualization for EDM/VJ applications");
            ctx.ui.add_space(16.0);

            // Generate sample waveform data (sine wave with harmonics)
            let time = ctx.ui.input(|i| i.time) as f32;
            let samples: Vec<f32> = (0..128)
                .map(|i| {
                    let t = i as f32 / 128.0 * std::f32::consts::PI * 4.0 + time * 2.0;
                    (t.sin() * 0.6 + (t * 2.0).sin() * 0.3 + (t * 3.0).sin() * 0.1)
                })
                .collect();

            // Line style (default)
            ctx.ui.label("Line style:");
            Waveform::new(&samples).height(60.0).show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Filled style
            ctx.ui.label("Filled style:");
            Waveform::new(&samples).height(60.0).filled().show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Bars style
            ctx.ui.label("Bars style:");
            Waveform::new(&samples).height(60.0).bars().show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Stereo with grid
            let samples_right: Vec<f32> = (0..128)
                .map(|i| {
                    let t = i as f32 / 128.0 * std::f32::consts::PI * 4.0 + time * 2.0 + 0.5;
                    (t.sin() * 0.5 + (t * 1.5).sin() * 0.4)
                })
                .collect();

            ctx.ui.label("Stereo with grid:");
            Waveform::stereo(&samples, &samples_right)
                .height(40.0)
                .grid(true)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"Waveform::new(&audio_samples)
    .height(60.0)
    .filled()
    .show(ctx.ui);"#).show(ctx.ui);
        }

        "Spectrum" => {
            ctx.ui.heading("Spectrum");
            ctx.ui.label("Frequency spectrum analyzer for EDM/VJ applications");
            ctx.ui.add_space(16.0);

            // Generate fake FFT data (simulated frequency bins)
            let time = ctx.ui.input(|i| i.time) as f32;
            let fft_bins: Vec<f32> = (0..64)
                .map(|i| {
                    let freq = i as f32 / 64.0;
                    let base = (1.0 - freq).powf(1.5); // Bass-heavy
                    let pulse = ((time * 2.0 + i as f32 * 0.1).sin() * 0.5 + 0.5);
                    (base * pulse * 0.8).clamp(0.0, 1.0)
                })
                .collect();

            // Solid color (default)
            ctx.ui.label("Solid color:");
            Spectrum::new(&fft_bins).height(80.0).show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Gradient
            ctx.ui.label("Gradient:");
            Spectrum::new(&fft_bins).height(80.0).gradient().show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Rainbow with peak hold
            ctx.ui.label("Rainbow with peak hold:");
            Spectrum::new(&fft_bins)
                .height(80.0)
                .rainbow()
                .peak_hold(true)
                .show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Mirrored (symmetric)
            ctx.ui.label("Mirrored:");
            Spectrum::new(&fft_bins)
                .height(80.0)
                .mirrored(true)
                .gradient()
                .show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"Spectrum::new(&fft_bins)
    .bands(32)
    .rainbow()
    .peak_hold(true)
    .show(ctx.ui);"#).show(ctx.ui);
        }

        "LevelMeter" => {
            ctx.ui.heading("LevelMeter");
            ctx.ui.label("VU/Peak meter for audio level visualization");
            ctx.ui.add_space(16.0);

            // Animated level values
            let time = ctx.ui.input(|i| i.time) as f32;
            let left_db = -60.0 + (time * 1.3).sin() * 30.0 + (time * 3.7).sin() * 15.0 + 30.0;
            let right_db = -60.0 + (time * 1.5).sin() * 30.0 + (time * 4.1).sin() * 15.0 + 28.0;
            let left_db = left_db.clamp(-60.0, 6.0);
            let right_db = right_db.clamp(-60.0, 6.0);

            // Peak hold (simulate)
            let left_peak = left_db + 3.0;
            let right_peak = right_db + 3.0;

            ctx.horizontal(|ctx| {
                ctx.vertical(|ctx| {
                    ctx.ui.label("Mono:");
                    LevelMeter::new()
                        .size(20.0, 180.0)
                        .show(ctx.ui, left_db);
                });

                ctx.ui.add_space(24.0);

                ctx.vertical(|ctx| {
                    ctx.ui.label("Stereo:");
                    LevelMeter::new()
                        .size(32.0, 180.0)
                        .stereo(true)
                        .show_stereo(ctx.ui, left_db, right_db);
                });

                ctx.ui.add_space(24.0);

                ctx.vertical(|ctx| {
                    ctx.ui.label("With peak hold:");
                    LevelMeter::new()
                        .size(32.0, 180.0)
                        .stereo(true)
                        .show_stereo_with_peak(ctx.ui, left_db, right_db, left_peak, right_peak);
                });

                ctx.ui.add_space(24.0);

                ctx.vertical(|ctx| {
                    ctx.ui.label("No scale:");
                    LevelMeter::new()
                        .size(20.0, 180.0)
                        .show_scale(false)
                        .show(ctx.ui, right_db);
                });
            });

            // Horizontal orientation
            ctx.ui.add_space(16.0);
            ctx.ui.label("Horizontal:");
            LevelMeter::new()
                .size(200.0, 20.0)
                .orientation(MeterOrientation::Horizontal)
                .show_scale(false)
                .show(ctx.ui, left_db);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"LevelMeter::new()
    .stereo(true)
    .show_stereo_with_peak(ui, left_db, right_db, left_peak, right_peak);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Green/Yellow/Red color zones");
            ctx.ui.label("• Configurable dB range");
            ctx.ui.label("• Peak hold indicator");
            ctx.ui.label("• Optional dB scale");
            ctx.ui.label("• Vertical or horizontal orientation");

            // Request repaint for animation
            ctx.ui.ctx().request_repaint();
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

        "Oscilloscope" => {
            ctx.ui.heading("Oscilloscope");
            ctx.ui.label("Real-time signal visualization with trigger and grid");
            ctx.ui.add_space(16.0);

            // Generate animated sine wave
            let time = ctx.ui.input(|i| i.time) as f32;
            let samples: Vec<f32> = (0..512)
                .map(|i| {
                    let t = i as f32 / 512.0 * std::f32::consts::PI * 8.0 + time * 3.0;
                    (t.sin() * 0.7 + (t * 3.0).sin() * 0.2).clamp(-1.0, 1.0)
                })
                .collect();

            // Secondary signal for XY mode
            let samples_y: Vec<f32> = (0..512)
                .map(|i| {
                    let t = i as f32 / 512.0 * std::f32::consts::PI * 8.0 + time * 3.0;
                    (t * 1.5).cos().clamp(-1.0, 1.0)
                })
                .collect();

            // Standard oscilloscope
            ctx.ui.label("Line mode:");
            Oscilloscope::new(&samples)
                .height(80.0)
                .trigger(TriggerMode::Rising)
                .show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Phosphor glow (CRT style)
            ctx.ui.label("Phosphor glow (retro CRT):");
            Oscilloscope::new(&samples)
                .height(80.0)
                .phosphor(true)
                .trigger(TriggerMode::Rising)
                .show(ctx.ui);

            ctx.ui.add_space(12.0);

            // Filled mode
            ctx.ui.label("Filled mode:");
            Oscilloscope::new(&samples)
                .height(80.0)
                .filled()
                .show(ctx.ui);

            ctx.ui.add_space(12.0);

            // XY mode (Lissajous)
            ctx.ui.label("XY mode (Lissajous):");
            Oscilloscope::new(&samples)
                .height(120.0)
                .xy(&samples_y)
                .phosphor(true)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"Oscilloscope::new(&signal_buffer)
    .trigger(TriggerMode::Rising)
    .phosphor(true)
    .show(ui);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Line, Filled, Dots, XY modes");
            ctx.ui.label("• Trigger modes (Free, Rising, Falling)");
            ctx.ui.label("• Phosphor glow effect (CRT style)");
            ctx.ui.label("• Configurable grid");

            // Request repaint for animation
            ctx.ui.ctx().request_repaint();
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

        "BpmDisplay" => {
            ctx.ui.heading("BpmDisplay");
            ctx.ui.label("Large numeric display for BPM/tempo (tap to interact)");
            ctx.ui.add_space(16.0);

            // Animated BPM for demo
            let time = ctx.ui.input(|i| i.time);
            let bpm = 120.0 + (time * 0.5).sin() * 8.0;

            ctx.horizontal(|ctx| {
                // Modern style
                ctx.vertical(|ctx| {
                    ctx.ui.label("Modern:");
                    BpmDisplay::new()
                        .label("BPM")
                        .show(ctx.ui, bpm);
                });

                ctx.ui.add_space(24.0);

                // Segment style (LED)
                ctx.vertical(|ctx| {
                    ctx.ui.label("Segment (LED):");
                    BpmDisplay::new()
                        .label("TEMPO")
                        .segment()
                        .show(ctx.ui, bpm);
                });

                ctx.ui.add_space(24.0);

                // Minimal
                ctx.vertical(|ctx| {
                    ctx.ui.label("Minimal:");
                    BpmDisplay::new()
                        .minimal()
                        .large()
                        .decimals(0)
                        .show(ctx.ui, bpm);
                });
            });

            ctx.ui.add_space(12.0);

            ctx.horizontal(|ctx| {
                // Compact
                ctx.vertical(|ctx| {
                    ctx.ui.label("Compact:");
                    BpmDisplay::new()
                        .compact()
                        .decimals(0)
                        .show(ctx.ui, 128.0);
                });

                ctx.ui.add_space(24.0);

                // Blinking (sync indicator)
                ctx.vertical(|ctx| {
                    ctx.ui.label("Blinking:");
                    BpmDisplay::new()
                        .segment()
                        .blinking(true)
                        .decimals(0)
                        .show(ctx.ui, 140.0);
                });
            });

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"BpmDisplay::new()
    .label("BPM")
    .segment()
    .show_with(ctx, model.bpm, || Msg::TapTempo);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Modern / Segment / Minimal styles");
            ctx.ui.label("• Size variants");
            ctx.ui.label("• Blinking effect for sync");
            ctx.ui.label("• Click for tap tempo");

            // Request repaint for animation
            ctx.ui.ctx().request_repaint();
        }

        "ClipGrid" => {
            ctx.ui.heading("ClipGrid");
            ctx.ui.label("Ableton Live style clip launcher grid");
            ctx.ui.add_space(16.0);

            // Sample clips
            let clips = vec![
                ClipCell::new("Intro"),
                ClipCell::new("Verse").with_color(egui::Color32::from_rgb(80, 180, 120)),
                ClipCell::new("Chorus").with_color(egui::Color32::from_rgb(220, 100, 100)),
                ClipCell::new("Bridge").with_color(egui::Color32::from_rgb(100, 150, 220)),
                ClipCell::new("Drop").with_color(egui::Color32::from_rgb(200, 120, 200)),
                ClipCell::new("Build").with_color(egui::Color32::from_rgb(220, 180, 80)),
                ClipCell::new("Break"),
                ClipCell::new("Outro"),
            ];

            // Animated states for demo
            let time = ctx.ui.input(|i| i.time);
            let current = ((time * 0.5) as usize) % clips.len();
            let queued = vec![(current + 1) % clips.len()];

            ctx.ui.label("4-column grid (with playing/queued state):");
            if let Some(_clicked) = ClipGrid::new(&clips, 4)
                .current(Some(current))
                .queued(&queued)
                .cell_size(80.0, 50.0)
                .show_index(true)
                .show(ctx.ui)
            {
                // Handle clip click
            }

            ctx.ui.add_space(16.0);

            ctx.ui.label("2-column grid (compact):");
            ClipGrid::new(&clips[..4], 2)
                .cell_size(100.0, 40.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"let clips = vec![
    ClipCell::new("Intro"),
    ClipCell::new("Verse").with_color(Color32::GREEN),
];

if let Some(idx) = ClipGrid::new(&clips, 4)
    .current(model.playing)
    .queued(&model.queue)
    .show(ctx.ui) {
    return Some(Msg::PlayClip(idx));
}"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Features:");
            ctx.ui.label("• Playing state with pulse animation");
            ctx.ui.label("• Queued state with indicator");
            ctx.ui.label("• Custom colors per clip");
            ctx.ui.label("• Configurable grid columns");
            ctx.ui.label("• Optional index display");

            ctx.ui.ctx().request_repaint();
        }

        "Plot" => {
            ctx.ui.heading("Plot");
            ctx.ui.label("egui_plot wrapper with theme integration");
            ctx.ui.add_space(16.0);

            // Generate sample data
            let time = ctx.ui.input(|i| i.time) as f64;

            // LinePlot - Waveform style
            ctx.ui.label("LinePlot (waveform):");
            let samples: Vec<f64> = (0..200)
                .map(|i| {
                    let t = i as f64 * 0.05 + time;
                    (t * 2.0).sin() * 0.5 + (t * 5.0).sin() * 0.3
                })
                .collect();
            LinePlot::new("waveform", &samples)
                .size(300.0, 80.0)
                .fill(true)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            // EnvelopePlot - ADSR
            ctx.ui.label("EnvelopePlot (ADSR):");
            let envelope = vec![
                (0.0, 0.0),   // Start
                (0.1, 1.0),   // Attack
                (0.3, 0.7),   // Decay -> Sustain
                (0.7, 0.7),   // Sustain
                (1.0, 0.0),   // Release
            ];
            EnvelopePlot::new("adsr", &envelope)
                .size(300.0, 80.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            // AutomationPlot
            ctx.ui.label("AutomationPlot (parameter):");
            let automation = vec![
                (0.0, 0.5),
                (0.2, 0.8),
                (0.4, 0.3),
                (0.6, 0.9),
                (0.8, 0.4),
                (1.0, 0.6),
            ];
            AutomationPlot::new("automation", &automation)
                .size(300.0, 60.0)
                .show_points(true)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            // FrequencyPlot - EQ curve
            ctx.ui.label("FrequencyPlot (EQ curve):");
            let eq_curve: Vec<(f64, f64)> = (0..100)
                .map(|i| {
                    let freq = 20.0 * (1000.0_f64).powf(i as f64 / 100.0);
                    // Simple low-shelf + high-shelf simulation
                    let db = if freq < 200.0 {
                        3.0
                    } else if freq > 8000.0 {
                        -2.0
                    } else {
                        0.0
                    };
                    (freq, db)
                })
                .collect();
            FrequencyPlot::new("eq", &eq_curve)
                .size(300.0, 100.0)
                .db_range(-12.0, 12.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);

            // BarPlot
            ctx.ui.label("BarPlot (spectrum bins):");
            let bars: Vec<f64> = (0..16)
                .map(|i| {
                    let t = time + i as f64 * 0.1;
                    (t * 2.0).sin().abs() * 0.8 + 0.2
                })
                .collect();
            BarPlot::new("bars", &bars)
                .size(300.0, 80.0)
                .show(ctx.ui);

            ctx.ui.add_space(16.0);
            ctx.ui.separator();
            ctx.ui.add_space(8.0);

            ctx.ui.label("Usage:");
            Code::new(r#"// Line plot
LinePlot::new("id", &samples)
    .fill(true)
    .show(ctx.ui);

// Envelope (ADSR)
EnvelopePlot::new("env", &[(0.0, 0.0), (0.1, 1.0), ...])
    .show(ctx.ui);

// Frequency response
FrequencyPlot::new("eq", &curve)
    .db_range(-24.0, 24.0)
    .show(ctx.ui);"#).show(ctx.ui);

            ctx.ui.add_space(8.0);
            ctx.ui.label("Plot types:");
            ctx.ui.label("• LinePlot - Waveforms, signals");
            ctx.ui.label("• EnvelopePlot - ADSR, modulation");
            ctx.ui.label("• AutomationPlot - DAW automation lanes");
            ctx.ui.label("• FrequencyPlot - EQ, spectrum");
            ctx.ui.label("• BarPlot - Histogram, spectrum bins");

            ctx.ui.ctx().request_repaint();
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
            ctx.ui.label("Data table component");
            ctx.ui.add_space(8.0);

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
