//! Atoms - Basic UI building blocks
//!
//! Atoms are the smallest UI components: buttons, inputs, badges, etc.
//!
//! # Creating New Atoms
//!
//! When creating a new atom component, ensure it respects theme scaling:
//!
//! ## Required: Use Theme Values
//!
//! ```ignore
//! use crate::Theme;
//!
//! impl Widget for MyAtom {
//!     fn ui(self, ui: &mut Ui) -> Response {
//!         let theme = Theme::current(ui.ctx());
//!
//!         // ✅ Good: Use theme values (respects scaling)
//!         let height = theme.spacing_md + theme.spacing_sm;
//!         let font = egui::FontId::proportional(theme.font_size_sm);
//!         let padding = theme.spacing_sm;
//!         let border = theme.stroke_width;
//!         let corner = theme.radius_sm;
//!
//!         // ❌ Bad: Fixed values (ignores scaling)
//!         let height = 24.0;
//!         let font = egui::FontId::proportional(14.0);
//!     }
//! }
//! ```
//!
//! ## Theme Token Reference
//!
//! | Category | Tokens | Scaled by |
//! |----------|--------|-----------|
//! | Spacing | `spacing_xs/sm/md/lg/xl` | `with_scale()`, `with_spacing_scale()` |
//! | Font | `font_size_xs/sm/md/lg/xl/2xl/3xl` | `with_font_scale()` |
//! | Radius | `radius_sm/md/lg` | `with_radius_scale()` |
//! | Stroke | `stroke_width`, `border_width` | `with_stroke_scale()` |
//!
//! ## Size Variants Pattern
//!
//! For components with size variants, combine spacing tokens:
//!
//! ```ignore
//! let height = match self.size {
//!     Size::Compact => theme.spacing_md + theme.spacing_sm,  // ~24px
//!     Size::Medium => theme.spacing_lg + theme.spacing_md,   // ~32px
//!     Size::Large => theme.spacing_xl + theme.spacing_md,    // ~40px
//! };
//! ```

mod arc_slider;
mod badge;
mod bpm_display;
mod button;
mod clip_grid;
mod button_group;
mod checkbox;
mod code;
mod context_menu;
mod fader;
mod icon;
mod input;
mod knob;
mod level_meter;
mod link;
mod oscilloscope;
#[cfg(feature = "plot")]
mod plot;
#[cfg(feature = "extras")]
mod extras;
mod list_item;
mod select;
mod slider;
mod spectrum;
mod text;
mod toggle;
mod tooltip;
mod validated_input;
mod waveform;
mod xypad;

pub use arc_slider::{ArcSlider, ArcSliderSize, ArcStyle};
pub use badge::{Badge, BadgeVariant};
pub use bpm_display::{BpmDisplay, DisplaySize, DisplayStyle};
pub use button::{Button, ButtonVariant};
pub use button_group::{ButtonGroup, GroupOrientation, GroupSize};
pub use clip_grid::{ClipCell, ClipGrid, ClipState};
pub use checkbox::Checkbox;
pub use code::{Code, CodeBlock};
pub use context_menu::{ContextMenuExt, ContextMenuItem};
pub use fader::{Fader, FaderSize};
pub use icon::{icons, Icon};
pub use input::Input;
pub use knob::{Knob, KnobSize};
pub use level_meter::{LevelMeter, MeterMode, MeterOrientation};
pub use link::Link;
pub use oscilloscope::{Oscilloscope, ScopeMode, TriggerMode};
#[cfg(feature = "plot")]
pub use plot::{AutomationPlot, BarPlot, EnvelopePlot, FrequencyPlot, LinePlot};
#[cfg(feature = "plot")]
pub use plot::raw as plot_raw;
#[cfg(feature = "extras")]
pub use extras::{ColumnWidth, Strip, StripDirection, StripSize};
#[cfg(feature = "extras")]
pub use extras::raw as extras_raw;
pub use list_item::{ListItem, ListItemSize};
pub use select::Select;
pub use slider::Slider;
pub use spectrum::{Spectrum, SpectrumColorMode};
pub use text::{MutedText, Text, TextSize};
pub use toggle::Toggle;
pub use tooltip::ResponseExt;
pub use validated_input::{ValidatedInput, ValidationState};
pub use waveform::{Waveform, WaveformStyle};
pub use xypad::XYPad;
