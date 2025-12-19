# egui-cha

A TEA (The Elm Architecture) framework for [egui](https://github.com/emilk/egui) with a built-in Design System.

## Why egui-cha?

Building admin panels, debug tools, or internal utilities with egui is great, but:

- **Time consuming** - Writing everything from scratch takes time you don't have
- **Prototypes grow fast** - "Quick hack" becomes 5000 lines of spaghetti
- **Separation gets blurry** - UI code and application logic become tangled

### The Problem with Raw egui

```rust
// Typical egui code - logic scattered everywhere
if ui.button("Save").clicked() {
    self.saving = true;
    self.data.validate();  // Business logic here?
    self.api.save(&self.data);  // Side effects here?
    self.saving = false;
    self.last_error = None;  // State management here?
}
```

### The TEA Solution

```rust
// View: purely presentational
Button::primary("Save").on_click(ctx, Msg::Save);

// Update: all logic in one place
fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
    match msg {
        Msg::Save => {
            model.saving = true;
            Cmd::try_task(
                api::save(&model.data),
                |_| Msg::SaveSuccess,
                |e| Msg::SaveError(e),
            )
        }
        Msg::SaveSuccess => { model.saving = false; Cmd::none() }
        Msg::SaveError(e) => { model.last_error = Some(e); Cmd::none() }
    }
}
```

**Result**: Logic stays in `update()`, views stay simple, and your "quick tool" won't fall apart as it grows.

## Features

- **TEA Architecture**: Model-View-Update pattern with `Cmd` for side effects
- **Router**: Built-in navigation with history (back/forward)
- **Design System**: Themed components following Atomic Design principles
- **Icon Support**: Phosphor Icons integration (embedded font)
- **Error Handling**: Structured error handling with `Cmd::try_task` and `ErrorConsole`
- **Testing Utilities**: `TestRunner` for unit testing your app logic

## Quick Start

```rust
use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;

fn main() -> eframe::Result<()> {
    egui_cha::run::<MyApp>(RunConfig::new("My App"))
}

struct MyApp;

#[derive(Default)]
struct Model {
    count: i32,
}

#[derive(Clone)]
enum Msg {
    Increment,
    Decrement,
}

impl App for MyApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        (Model::default(), Cmd::none())
    }

    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Increment => model.count += 1,
            Msg::Decrement => model.count -= 1,
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        ctx.ui.heading("Counter");
        ctx.ui.label(format!("Count: {}", model.count));

        ctx.horizontal(|ctx| {
            Button::primary("+").on_click(ctx, Msg::Increment);
            Button::secondary("-").on_click(ctx, Msg::Decrement);
        });
    }
}
```

## Architecture

```
┌────────────────────────────────────────┐
│  egui-cha-ds (Design System)           │
│  Button, Input, Card, Icon, Theme...   │
├────────────────────────────────────────┤
│  egui-cha (TEA Core)                   │
│  App, Cmd, ViewCtx, Router, Component  │
└────────────────────────────────────────┘
                   ↓
                 egui
```

## Components

### Core (egui-cha)

| Component | Description |
|-----------|-------------|
| `App` | Main application trait with `init`, `update`, `view` |
| `Cmd` | Side effects: `task`, `delay`, `try_task`, `from_result` |
| `ViewCtx` | UI context with `emit`, `horizontal`, `vertical`, `group` |
| `Router` | Page navigation with history stack |
| `Component` | Reusable component trait |

### Design System (egui-cha-ds)

#### Atoms
| Component | Variants |
|-----------|----------|
| `Button` | Primary, Secondary, Outline, Ghost, Danger |
| `Badge` | Default, Success, Warning, Error, Info |
| `Input` | Text input with TEA-style callbacks |
| `Icon` | Phosphor Icons (house, gear, info, etc.) |

#### Molecules
| Component | Description |
|-----------|-------------|
| `Card` | Container with optional title |
| `Navbar` | Horizontal navigation bar |
| `sidebar` | Vertical navigation |
| `ErrorConsole` | Error/Warning/Info message display |
| `SearchBar` | Search input with submit |

#### Theme
```rust
// Apply theme to egui context
let theme = Theme::dark(); // or Theme::light()
theme.apply(ctx.ui.ctx());
```

## Commands (Side Effects)

```rust
// Async task
Cmd::task(async {
    let data = fetch_data().await;
    Msg::DataLoaded(data)
})

// Delayed message
Cmd::delay(Duration::from_secs(1), Msg::Tick)

// Fallible async with error handling
Cmd::try_task(
    fetch_data(),
    |data| Msg::Success(data),
    |err| Msg::Error(err.to_string()),
)

// Batch multiple commands
Cmd::batch([cmd1, cmd2, cmd3])
```

## Router

```rust
#[derive(Clone, PartialEq, Default)]
enum Page {
    #[default]
    Home,
    Settings,
    Profile(u64),
}

// In your Model
struct Model {
    router: Router<Page>,
}

// Navigate
Msg::Router(RouterMsg::Navigate(Page::Settings))
Msg::Router(RouterMsg::Back)
Msg::Router(RouterMsg::Forward)

// In view
navbar(ctx, &model.router, &[
    ("Home", Page::Home),
    ("Settings", Page::Settings),
], Msg::Router);
```

## Icons (Phosphor)

```rust
use egui_cha_ds::atoms::{Icon, icons};

// Using convenience constructors
Icon::house().size(20.0).show(ctx.ui);
Icon::gear().color(Color32::RED).show(ctx.ui);

// Using icon constants directly
Icon::new(icons::CHECK).show(ctx.ui);
```

Available icons: `HOUSE`, `GEAR`, `HASH`, `INFO`, `USER`, `CHECK`, `WARNING`, `PLUS`, `MINUS`, `X`, `ARROW_LEFT`, `ARROW_RIGHT`

## Testing

```rust
use egui_cha::test_prelude::*;

#[test]
fn test_increment() {
    let mut runner = TestRunner::<MyApp>::new();

    runner.send(Msg::Increment);
    assert_eq!(runner.model().count, 1);
    assert!(runner.last_was_none()); // No side effects
}

#[test]
fn test_async_command() {
    let mut runner = TestRunner::<MyApp>::new();

    runner.send(Msg::FetchData);
    assert!(runner.last_was_task()); // Returned an async task
}
```

## License

MIT

## Credits

- [egui](https://github.com/emilk/egui) - Immediate mode GUI library
- [Phosphor Icons](https://phosphoricons.com/) - Icon font
