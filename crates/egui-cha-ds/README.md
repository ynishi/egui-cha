# egui-cha-ds

Design System for [egui-cha](https://github.com/ynishi/egui-cha) - a TEA (The Elm Architecture) framework for egui.

## Overview

`egui-cha-ds` provides themed UI components following Atomic Design principles:

- **Atoms**: Basic building blocks (Button, Input, Badge, Icon, Slider, etc.)
- **Molecules**: Combinations of atoms (Card, Tabs, Modal, Form, Toast, etc.)
- **Theme**: Consistent styling across all components

## Installation

```bash
cargo add egui-cha-ds
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
egui-cha-ds = "0.1"
```

## Features

| Feature | Description |
|---------|-------------|
| `macros` | Layout DSL via `cha!` macro |
| `plot` | Chart/plotting components |
| `extras` | Extended components |
| `dock` | Docking panel system |
| `snarl` | Node graph editor |
| `audio` | Audio visualization (BPM, transport, waveform) |
| `midi` | MIDI components (keyboard, piano roll, mapper) |
| `mixer` | Mixer components (channel strip, effects, automation) |
| `visual` | Visual editing (layers, masks, color wheel) |
| `studio` | All VJ/DAW components (default) |

## Quick Example

```rust
use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;

fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
    // Apply theme
    Theme::dark().apply(ctx.ui.ctx());

    // Use components
    Button::primary("Save").on_click(ctx, Msg::Save);

    Card::new("Settings").show(ctx, |ctx| {
        Input::new()
            .placeholder("Enter name")
            .show_with(ctx, &model.name, Msg::NameChanged);
    });
}
```

## Components

### Atoms (13+ components)

Button, Badge, Text, Input, ValidatedInput, Checkbox, Toggle, Slider, Select, Icon, Link, Code, Tooltip, ContextMenu

### Molecules (9+ components)

Card, Tabs, Modal, Table, Navbar, ErrorConsole, Toast, Form, SearchBar

### VJ/DAW Components

When `studio` feature is enabled:
- **Audio**: BPM display, Transport, Waveform, Oscilloscope, Spectrum, Level meter
- **MIDI**: Keyboard, Piano roll, MIDI mapper, MIDI monitor
- **Mixer**: Channel strip, Crossfader, Envelope editor, Automation lane, Effect rack
- **Visual**: Timeline, Layer stack, Color wheel, Gradient editor, Transform gizmo

## Documentation

For full documentation, see the [main repository](https://github.com/ynishi/egui-cha).

## License

MIT OR Apache-2.0
