//! Counter example demonstrating egui-cha TEA architecture with Router

use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    egui_cha::run::<CounterApp>(
        RunConfig::new("egui-cha Router Demo")
            .with_size(600.0, 500.0),
    )
}

// ============================================================
// Pages
// ============================================================

#[derive(Clone, PartialEq, Default, Debug)]
enum Page {
    #[default]
    Home,
    Counter,
    Settings,
    About,
}

// ============================================================
// App
// ============================================================

struct CounterApp;

#[derive(Default)]
struct Model {
    router: Router<Page>,
    // Page-specific state
    counter: CounterState,
    settings: SettingsState,
    // Global
    theme: Theme,
    errors: ErrorConsoleState,
}

#[derive(Default)]
struct CounterState {
    count: i32,
    history: Vec<i32>,
}

#[derive(Default)]
struct SettingsState {
    username: String,
    notifications: bool,
}

#[derive(Clone, Debug)]
enum Msg {
    // Router
    Router(RouterMsg<Page>),
    // Counter page
    Increment,
    Decrement,
    Reset,
    AddToHistory,
    DelayedIncrement,
    // Settings page
    UsernameChanged(String),
    ToggleNotifications,
    // Global
    ToggleTheme,
    ErrorConsole(ErrorConsoleMsg),
}

impl App for CounterApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        (
            Model {
                router: Router::new(Page::Home),
                theme: Theme::light(),
                errors: ErrorConsoleState::new().with_max_entries(3),
                ..Default::default()
            },
            Cmd::none(),
        )
    }

    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        model.errors.cleanup();

        match msg {
            // Router
            Msg::Router(router_msg) => {
                model.router.handle(router_msg);
            }
            // Counter
            Msg::Increment => model.counter.count += 1,
            Msg::Decrement => model.counter.count -= 1,
            Msg::Reset => model.counter.count = 0,
            Msg::AddToHistory => {
                model.counter.history.push(model.counter.count);
            }
            Msg::DelayedIncrement => {
                return Cmd::delay(std::time::Duration::from_secs(1), Msg::Increment);
            }
            // Settings
            Msg::UsernameChanged(name) => model.settings.username = name,
            Msg::ToggleNotifications => {
                model.settings.notifications = !model.settings.notifications;
            }
            // Global
            Msg::ToggleTheme => {
                model.theme = match model.theme.variant {
                    ThemeVariant::Light => Theme::dark(),
                    ThemeVariant::Dark => Theme::light(),
                };
            }
            Msg::ErrorConsole(console_msg) => match console_msg {
                ErrorConsoleMsg::Dismiss(i) => model.errors.dismiss(i),
                ErrorConsoleMsg::DismissAll => model.errors.clear(),
            },
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        model.theme.apply(ctx.ui.ctx());

        // Navigation bar (icons rendered separately in nav items)
        navbar(
            ctx,
            &model.router,
            &[
                ("Home", Page::Home),
                ("Counter", Page::Counter),
                ("Settings", Page::Settings),
                ("About", Page::About),
            ],
            Msg::Router,
        );

        ctx.ui.add_space(8.0);

        // Error console
        ErrorConsole::show(ctx, &model.errors, Msg::ErrorConsole);
        if !model.errors.is_empty() {
            ctx.ui.add_space(8.0);
        }

        // Page content
        match model.router.current() {
            Page::Home => home_page(model, ctx),
            Page::Counter => counter_page(model, ctx),
            Page::Settings => settings_page(model, ctx),
            Page::About => about_page(model, ctx),
        }

        // Footer
        ctx.ui.add_space(16.0);
        ctx.ui.separator();
        ctx.horizontal(|ctx| {
            Button::ghost("Toggle Theme").on_click(ctx, Msg::ToggleTheme);
            ctx.ui.label(format!(
                "History: {} pages",
                model.router.history_len()
            ));
        });
    }
}

// ============================================================
// Page views
// ============================================================

fn home_page(_model: &Model, ctx: &mut ViewCtx<Msg>) {
    ctx.ui.heading("Welcome to egui-cha!");
    ctx.ui.add_space(8.0);

    ctx.ui.label("This demo showcases:");
    ctx.ui.label("  TEA (The Elm Architecture) pattern");
    ctx.ui.label("  Router with history");
    ctx.ui.label("  Design System components");
    ctx.ui.label("  Error handling");

    ctx.ui.add_space(16.0);

    // Icon demo
    ctx.ui.label("Phosphor Icons:");
    ctx.horizontal(|ctx| {
        Icon::house().size(20.0).show(ctx.ui);
        Icon::gear().size(20.0).show(ctx.ui);
        Icon::hash().size(20.0).show(ctx.ui);
        Icon::info().size(20.0).show(ctx.ui);
        Icon::user().size(20.0).show(ctx.ui);
        Icon::check().size(20.0).show(ctx.ui);
        Icon::warning().size(20.0).show(ctx.ui);
        Icon::plus().size(20.0).show(ctx.ui);
        Icon::minus().size(20.0).show(ctx.ui);
    });

    ctx.ui.add_space(16.0);

    ctx.horizontal(|ctx| {
        Button::primary("Go to Counter").on_click(ctx, Msg::Router(RouterMsg::Navigate(Page::Counter)));
        Button::secondary("Settings").on_click(ctx, Msg::Router(RouterMsg::Navigate(Page::Settings)));
    });
}

fn counter_page(model: &Model, ctx: &mut ViewCtx<Msg>) {
    Card::titled("Counter").show_ctx(ctx, |ctx| {
        ctx.ui.label(format!("Count: {}", model.counter.count));

        ctx.horizontal(|ctx| {
            Button::primary("+").on_click(ctx, Msg::Increment);
            Button::secondary("-").on_click(ctx, Msg::Decrement);
            Button::outline("Reset").on_click(ctx, Msg::Reset);
        });

        ctx.ui.add_space(8.0);

        ctx.horizontal(|ctx| {
            Button::ghost("Add to History").on_click(ctx, Msg::AddToHistory);
            Button::primary("+1 (1s delay)").on_click(ctx, Msg::DelayedIncrement);
        });
    });

    ctx.ui.add_space(16.0);

    Card::titled("History").show_ctx(ctx, |ctx| {
        if model.counter.history.is_empty() {
            ctx.ui.label("No history yet");
        } else {
            ctx.horizontal(|ctx| {
                for &value in &model.counter.history {
                    if value >= 0 {
                        Badge::success(&format!("{}", value)).show(ctx.ui);
                    } else {
                        Badge::error(&format!("{}", value)).show(ctx.ui);
                    }
                }
            });
        }
    });
}

fn settings_page(model: &Model, ctx: &mut ViewCtx<Msg>) {
    Card::titled("Settings").show_ctx(ctx, |ctx| {
        ctx.ui.label("Username:");
        Input::new()
            .placeholder("Enter username")
            .show_with(ctx, &model.settings.username, Msg::UsernameChanged);

        ctx.ui.add_space(8.0);

        ctx.horizontal(|ctx| {
            ctx.ui.checkbox(&mut model.settings.notifications.clone(), "Enable notifications");
            if ctx.ui.button("Toggle").clicked() {
                ctx.emit(Msg::ToggleNotifications);
            }
        });

        if !model.settings.username.is_empty() {
            ctx.ui.add_space(8.0);
            ctx.ui.label(format!("Hello, {}!", model.settings.username));
        }
    });
}

fn about_page(_model: &Model, ctx: &mut ViewCtx<Msg>) {
    Card::titled("About").show_ctx(ctx, |ctx| {
        ctx.ui.label("egui-cha v0.1.0");
        ctx.ui.add_space(8.0);
        ctx.ui.label("A TEA (The Elm Architecture) framework for egui.");
        ctx.ui.add_space(8.0);
        ctx.ui.label("Features:");
        ctx.ui.label("• Model-View-Update pattern");
        ctx.ui.label("• Cmd for side effects");
        ctx.ui.label("• Router with history");
        ctx.ui.label("• Design System");
        ctx.ui.label("• Error handling");
        ctx.ui.label("• Testing utilities");
    });

    ctx.ui.add_space(16.0);

    ctx.horizontal(|ctx| {
        Icon::arrow_left().size(14.0).show(ctx.ui);
        Button::outline("Back to Home").on_click(ctx, Msg::Router(RouterMsg::Navigate(Page::Home)));
    });
}
