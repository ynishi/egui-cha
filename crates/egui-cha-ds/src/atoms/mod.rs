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

mod badge;
mod button;
mod checkbox;
mod code;
mod context_menu;
mod icon;
mod input;
mod link;
mod list_item;
mod select;
mod slider;
mod text;
mod toggle;
mod tooltip;
mod validated_input;

pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
pub use code::{Code, CodeBlock};
pub use context_menu::{ContextMenuExt, ContextMenuItem};
pub use icon::{icons, Icon};
pub use input::Input;
pub use link::Link;
pub use list_item::{ListItem, ListItemSize};
pub use select::Select;
pub use slider::Slider;
pub use text::{MutedText, Text, TextSize};
pub use toggle::Toggle;
pub use tooltip::ResponseExt;
pub use validated_input::{ValidatedInput, ValidationState};
