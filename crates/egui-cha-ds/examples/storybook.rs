//! DS Storybook - Component showcase
//!
//! Visual catalog of all DS components

use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;
use egui_cha_ds::ConfirmResult;

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
}

const CATEGORIES: &[&str] = &["Atoms", "Molecules"];

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
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        model.theme.apply(ctx.ui.ctx());

        // Header
        ctx.horizontal(|ctx| {
            ctx.ui.heading("DS Storybook");
        });
        // Theme toggle button (separate to avoid borrow issues)
        let theme_label = if model.theme.variant == ThemeVariant::Dark { "Light Mode" } else { "Dark Mode" };
        Button::ghost(theme_label).on_click(ctx, Msg::ToggleTheme);
        ctx.ui.separator();

        // Main layout
        ctx.ui.columns(2, |columns| {
            // Left: Navigation
            columns[0].set_max_width(200.0);
            columns[0].heading("Components");
            columns[0].separator();

            // Category tabs
            for (i, cat) in CATEGORIES.iter().enumerate() {
                let selected = model.active_category == i;
                if columns[0].selectable_label(selected, *cat).clicked() {
                    // Can't emit in columns, will use workaround
                }
            }

            columns[0].separator();

            // Component list
            let components = if model.active_category == 0 { ATOMS } else { MOLECULES };
            for (i, comp) in components.iter().enumerate() {
                let selected = model.active_component == i;
                if columns[0].selectable_label(selected, format!("  {}", comp)).clicked() {
                    // Can't emit in columns
                }
            }

            // Right: Component preview
            columns[1].heading("Preview");
            columns[1].separator();
        });

        // Component selection via ctx (workaround for columns)
        ctx.ui.add_space(8.0);
        ctx.horizontal(|ctx| {
            ctx.ui.label("Category:");
            for (i, cat) in CATEGORIES.iter().enumerate() {
                if model.active_category == i {
                    Button::primary(*cat).on_click(ctx, Msg::SetCategory(i));
                } else {
                    Button::ghost(*cat).on_click(ctx, Msg::SetCategory(i));
                }
            }

            ctx.ui.add_space(16.0);
            ctx.ui.label("Component:");
            let components = if model.active_category == 0 { ATOMS } else { MOLECULES };
            for (i, comp) in components.iter().enumerate() {
                if model.active_component == i {
                    Button::secondary(*comp).on_click(ctx, Msg::SetComponent(i));
                } else {
                    Button::outline(*comp).on_click(ctx, Msg::SetComponent(i));
                }
            }
        });

        ctx.ui.separator();

        // Component preview
        Card::new().show_ctx(ctx, |ctx| {
            if model.active_category == 0 {
                render_atom(model, ctx);
            } else {
                render_molecule(model, ctx);
            }
        });

        // Modals
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

        _ => {
            ctx.ui.label("Component not implemented");
        }
    }
}
