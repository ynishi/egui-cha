//! Atoms - Basic UI building blocks
//!
//! Atoms are the smallest UI components organized by category:
//!
//! - **core**: Basic UI components (always available)
//! - **audio**: Audio visualization and control (requires `audio` feature)
//! - **midi**: MIDI input and editing (requires `midi` feature)
//! - **mixer**: Audio mixing and effects (requires `mixer` feature)
//! - **visual**: Video/graphics editing (requires `visual` feature)
//!
//! # Feature Flags
//!
//! - `audio` - Audio visualization components (BPM, transport, waveform, etc.)
//! - `midi` - MIDI components (keyboard, piano roll, mapper)
//! - `mixer` - Mixer components (channel strip, effects, automation)
//! - `visual` - Visual editing components (layers, masks, color wheel)
//! - `vj` - Enables `audio` + `visual`
//! - `daw` - Enables `audio` + `midi` + `mixer`
//! - `studio` - Enables all VJ/DAW components
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
//!         // Use theme values (respects scaling)
//!         let height = theme.spacing_md + theme.spacing_sm;
//!         let font = egui::FontId::proportional(theme.font_size_sm);
//!         let padding = theme.spacing_sm;
//!         let border = theme.stroke_width;
//!         let corner = theme.radius_sm;
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

// =============================================================================
// Core atoms (always available)
// =============================================================================
mod core;
pub use core::*;

// =============================================================================
// Audio atoms (audio feature)
// =============================================================================
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "audio")]
pub use audio::*;

// =============================================================================
// MIDI atoms (midi feature)
// =============================================================================
#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "midi")]
pub use midi::*;

// =============================================================================
// Mixer atoms (mixer feature)
// =============================================================================
#[cfg(feature = "mixer")]
pub mod mixer;
#[cfg(feature = "mixer")]
pub use mixer::*;

// =============================================================================
// Visual atoms (visual feature)
// =============================================================================
#[cfg(feature = "visual")]
pub mod visual;
#[cfg(feature = "visual")]
pub use visual::*;

// =============================================================================
// Plot atoms (plot feature - existing)
// =============================================================================
#[cfg(feature = "plot")]
mod plot;
#[cfg(feature = "plot")]
pub use plot::raw as plot_raw;
#[cfg(feature = "plot")]
pub use plot::{AutomationPlot, BarPlot, EnvelopePlot, FrequencyPlot, LinePlot, Sparkline};

// =============================================================================
// Extras atoms (extras feature - existing)
// =============================================================================
#[cfg(feature = "extras")]
mod extras;
#[cfg(feature = "extras")]
pub use extras::raw as extras_raw;
#[cfg(feature = "extras")]
pub use extras::{ColumnWidth, Strip, StripDirection, StripSize};
