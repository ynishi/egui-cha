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

## Installation

```bash
cargo add egui-cha egui-cha-ds
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
egui-cha = "0.1"
egui-cha-ds = "0.1"
```

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
| `ScrollArea` | Scrollable container with builder pattern |

### Design System (egui-cha-ds)

#### Theme
```rust
// Apply theme to egui context
let theme = Theme::dark(); // or Theme::light(), Theme::pastel(), Theme::pastel_dark()
theme.apply(ctx.ui.ctx());
```

Theme includes:
- **Colors**: Primary, Secondary, UI State (state_success/warning/danger/info), Log Severity (log_debug/info/warn/error/critical), Background, Text, Border
- **Spacing**: xs (4px), sm (8px), md (16px), lg (24px), xl (32px)
- **Border Radius**: sm, md, lg
- **Typography**: font_size_xs (10px) through font_size_3xl (30px)

#### Atoms (13 components)
| Component | Description |
|-----------|-------------|
| `Button` | Primary, Secondary, Outline, Ghost, Danger, Warning, Success, Info |
| `Badge` | Default, Success, Warning, Error, Info |
| `Text` | Typography: h1, h2, h3, body, small, caption with modifiers |
| `Input` | Text input with TEA-style callbacks |
| `ValidatedInput` | Input with validation state |
| `Checkbox` | Boolean toggle with label |
| `Toggle` | Switch-style boolean toggle |
| `Slider` | Numeric range input |
| `Select` | Dropdown selection |
| `Icon` | Phosphor Icons (house, gear, info, etc.) |
| `Link` | Hyperlink component |
| `Code` | Code block display |
| `Tooltip` | Themed tooltips via `ResponseExt` trait |
| `ContextMenu` | Right-click menu via `ContextMenuExt` trait |

#### Molecules (9 components)
| Component | Description |
|-----------|-------------|
| `Card` | Container with optional title |
| `Tabs` | Tabbed navigation with TabPanel |
| `Modal` | Dialog overlay |
| `Table` | Data table component |
| `Navbar` | Horizontal navigation bar |
| `ErrorConsole` | Error/Warning/Info message display |
| `Toast` | Temporary notifications with auto-dismiss |
| `Form` | Structured form with validation |
| `SearchBar` | Search input with submit |

#### Semantics (Pre-defined buttons)
```rust
// Consistent labels & icons across your app
semantics::save(ButtonStyle::Both).on_click(ctx, Msg::Save);
semantics::delete(ButtonStyle::Icon).on_click(ctx, Msg::Delete);
```
Available: `save`, `edit`, `delete`, `close`, `add`, `remove`, `search`, `refresh`, `play`, `pause`, `stop`, `settings`, `back`, `forward`, `confirm`, `cancel`, `copy`

## Layout DSL (cha! macro)

Declarative layout syntax for composing views:

```rust
use egui_cha_ds::cha;

cha!(ctx, {
    Col(spacing: 8.0) {
        Card("Settings") {
            Row {
                @gear(20.0)
                ctx.ui.label("Options")
            }
            Button::primary("Save").on_click(ctx, Msg::Save)
        }

        Scroll(max_height: 300.0) {
            // scrollable content
        }
    }
});
```

### Supported Nodes
| Node | Description |
|------|-------------|
| `Col` | Vertical layout (`ctx.vertical`) |
| `Row` | Horizontal layout (`ctx.horizontal`) |
| `Group` | Grouped layout (`ctx.group`) |
| `Scroll` | Vertical scroll area |
| `ScrollH` | Horizontal scroll area |
| `ScrollBoth` | Both directions scroll |
| `Card("title")` | Card container |
| `@icon` | Icon shorthand (e.g., `@house`, `@gear(20.0)`) |

### Properties
- **Layout**: `spacing`, `padding`
- **Scroll**: `max_height`, `max_width`, `min_height`, `min_width`, `id`
- **Card**: `padding`

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
