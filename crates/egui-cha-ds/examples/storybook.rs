//! DS Storybook - Component showcase
//!
//! Visual catalog of all DS components and framework features

use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;
use egui_cha_ds::ConfirmResult;
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
    input_value: String,
    select_value: usize,

    // Modal state
    show_modal: bool,
    show_confirm: bool,
    confirm_result: Option<bool>,

    // Theme
    theme: Theme,

    // Tabs demo
    tabs_index: usize,

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

    // Input
    InputChanged(String),

    // Select
    SelectChanged(usize),

    // Modal
    OpenModal,
    CloseModal,
    OpenConfirm,
    ConfirmResult(bool),

    // Theme
    ToggleTheme,

    // Tabs
    TabChanged(usize),

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
}

const CATEGORIES: &[&str] = &["Atoms", "Molecules", "Framework"];

const ATOMS: &[&str] = &[
    "Button",
    "Badge",
    "Icon",
    "Input",
    "Checkbox",
    "Toggle",
    "Slider",
    "Link",
    "Code",
];

const MOLECULES: &[&str] = &[
    "Card",
    "Tabs",
    "Modal",
    "Table",
    "Navbar",
    "ErrorConsole",
    "Columns",
];

const FRAMEWORK: &[&str] = &[
    "Cmd::delay",
    "Cmd::timeout",
    "Cmd::retry",
    "Sub::interval",
    "Debouncer",
    "Throttler",
];

impl App for StorybookApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        (
            Model {
                slider_value: 50.0,
                input_value: "Hello".to_string(),
                table_data: vec![
                    ("Alice".to_string(), 25, true),
                    ("Bob".to_string(), 30, false),
                    ("Carol".to_string(), 28, true),
                ],
                theme: Theme::light(),
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
            Msg::InputChanged(v) => {
                model.input_value = v;
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
                model.theme = match model.theme.variant {
                    ThemeVariant::Light => Theme::dark(),
                    ThemeVariant::Dark => Theme::light(),
                };
            }
            Msg::TabChanged(idx) => {
                model.tabs_index = idx;
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

                let theme_label = if model.theme.variant == ThemeVariant::Dark { "Light" } else { "Dark" };
                Button::ghost(theme_label).on_click(ctx, Msg::ToggleTheme);

                ctx.ui.separator();
                ctx.ui.heading("Components");
                ctx.ui.separator();

                // Category selection
                for (i, cat) in CATEGORIES.iter().enumerate() {
                    if model.active_category == i {
                        Button::primary(*cat).on_click(ctx, Msg::SetCategory(i));
                    } else {
                        Button::ghost(*cat).on_click(ctx, Msg::SetCategory(i));
                    }
                }

                ctx.ui.separator();

                // Component list
                let components = match model.active_category {
                    0 => ATOMS,
                    1 => MOLECULES,
                    _ => FRAMEWORK,
                };
                for (i, comp) in components.iter().enumerate() {
                    if model.active_component == i {
                        Button::secondary(*comp).on_click(ctx, Msg::SetComponent(i));
                    } else {
                        Button::outline(*comp).on_click(ctx, Msg::SetComponent(i));
                    }
                }
            },
            // Main: Component preview
            |ctx| {
                ctx.ui.heading("Preview");
                ctx.ui.separator();

                Card::new().show_ctx(ctx, |ctx| {
                    match model.active_category {
                        0 => render_atom(model, ctx),
                        1 => render_molecule(model, ctx),
                        _ => render_framework(model, ctx),
                    }
                });

                // Modals (inside main panel)
                if model.show_modal {
                    let close = Modal::titled("Demo Modal")
                        .show(ctx.ui, true, |ui| {
                            ui.label("This is a modal dialog.");
                            ui.label("You can put any content here.");
                            ui.add_space(16.0);
                            if ui.button("Close").clicked() {
                                // handled by close_requested
                            }
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
            ctx.ui.label("Error message display (see counter example)");
            ctx.ui.add_space(8.0);
            ctx.ui.label("The ErrorConsole component displays dismissible error messages.");
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

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}
