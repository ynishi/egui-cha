//! ScrollArea - Scrollable container with configurable options
//!
//! Provides a configurable scroll area builder that integrates with ViewCtx.
//!
//! # Examples
//!
//! ```rust
//! use egui_cha::ScrollArea;
//!
//! // Vertical scroll (default)
//! ScrollArea::vertical()
//!     .max_height(300.0)
//!     .show_ctx(ctx, |ctx| {
//!         for i in 0..100 {
//!             ctx.ui.label(format!("Item {}", i));
//!         }
//!     });
//!
//! // Horizontal scroll
//! ScrollArea::horizontal()
//!     .show_ctx(ctx, |ctx| {
//!         ctx.horizontal(|ctx| {
//!             for i in 0..20 {
//!                 ctx.ui.label(format!("{}", i));
//!             }
//!         });
//!     });
//!
//! // Both directions
//! ScrollArea::both()
//!     .max_height(400.0)
//!     .max_width(600.0)
//!     .show_ctx(ctx, |ctx| {
//!         // Large content
//!     });
//! ```

use egui::scroll_area::{ScrollBarVisibility, ScrollSource};
use egui::Ui;

use crate::ViewCtx;

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

/// A configurable scroll area builder
#[derive(Clone)]
pub struct ScrollArea {
    direction: ScrollDirection,
    id_salt: Option<egui::Id>,
    max_height: Option<f32>,
    max_width: Option<f32>,
    min_scrolled_height: Option<f32>,
    min_scrolled_width: Option<f32>,
    auto_shrink: [bool; 2],
    scroll_bar_visibility: ScrollBarVisibility,
    animated: bool,
    enable_scrolling: bool,
    scroll_offset: Option<egui::Vec2>,
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self {
            direction: ScrollDirection::Vertical,
            id_salt: None,
            max_height: None,
            max_width: None,
            min_scrolled_height: None,
            min_scrolled_width: None,
            auto_shrink: [true; 2],
            scroll_bar_visibility: ScrollBarVisibility::VisibleWhenNeeded,
            animated: true,
            enable_scrolling: true,
            scroll_offset: None,
        }
    }
}

impl ScrollArea {
    /// Create a new scroll area (vertical by default)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a vertical scroll area
    pub fn vertical() -> Self {
        Self {
            direction: ScrollDirection::Vertical,
            ..Default::default()
        }
    }

    /// Create a horizontal scroll area
    pub fn horizontal() -> Self {
        Self {
            direction: ScrollDirection::Horizontal,
            ..Default::default()
        }
    }

    /// Create a scroll area that scrolls both directions
    pub fn both() -> Self {
        Self {
            direction: ScrollDirection::Both,
            ..Default::default()
        }
    }

    /// Set a custom ID to avoid ID clashes
    pub fn id_salt(mut self, id: impl std::hash::Hash) -> Self {
        self.id_salt = Some(egui::Id::new(id));
        self
    }

    /// Set maximum height (pixels)
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    /// Set maximum width (pixels)
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Set minimum scrolled height
    pub fn min_scrolled_height(mut self, height: f32) -> Self {
        self.min_scrolled_height = Some(height);
        self
    }

    /// Set minimum scrolled width
    pub fn min_scrolled_width(mut self, width: f32) -> Self {
        self.min_scrolled_width = Some(width);
        self
    }

    /// Set auto-shrink behavior [horizontal, vertical]
    ///
    /// If `true`, the scroll area will shrink to fit its content.
    /// Default is `[true, true]`.
    pub fn auto_shrink(mut self, auto_shrink: [bool; 2]) -> Self {
        self.auto_shrink = auto_shrink;
        self
    }

    /// Disable auto-shrink (expand to fill available space)
    pub fn no_shrink(mut self) -> Self {
        self.auto_shrink = [false, false];
        self
    }

    /// Set scroll bar visibility
    pub fn scroll_bar_visibility(mut self, visibility: ScrollBarVisibility) -> Self {
        self.scroll_bar_visibility = visibility;
        self
    }

    /// Always show scroll bars
    pub fn always_show_scroll(mut self) -> Self {
        self.scroll_bar_visibility = ScrollBarVisibility::AlwaysVisible;
        self
    }

    /// Never show scroll bars
    pub fn hide_scroll(mut self) -> Self {
        self.scroll_bar_visibility = ScrollBarVisibility::AlwaysHidden;
        self
    }

    /// Enable or disable animated scrolling
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Enable or disable scrolling entirely
    pub fn enable_scrolling(mut self, enable: bool) -> Self {
        self.enable_scrolling = enable;
        self
    }

    /// Set initial scroll offset
    pub fn scroll_offset(mut self, offset: egui::Vec2) -> Self {
        self.scroll_offset = Some(offset);
        self
    }

    /// Show the scroll area with ViewCtx integration
    pub fn show_ctx<Msg, R>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        let area = self.build();
        ctx.scroll_area_with(|_| area, f)
    }

    /// Show the scroll area with raw egui::Ui
    pub fn show<R>(
        self,
        ui: &mut Ui,
        f: impl FnOnce(&mut Ui) -> R,
    ) -> egui::scroll_area::ScrollAreaOutput<R> {
        self.build().show(ui, f)
    }

    /// Build the underlying egui::ScrollArea
    fn build(self) -> egui::ScrollArea {
        let mut area = match self.direction {
            ScrollDirection::Vertical => egui::ScrollArea::vertical(),
            ScrollDirection::Horizontal => egui::ScrollArea::horizontal(),
            ScrollDirection::Both => egui::ScrollArea::both(),
        };

        if let Some(id) = self.id_salt {
            area = area.id_salt(id);
        }

        if let Some(h) = self.max_height {
            area = area.max_height(h);
        }

        if let Some(w) = self.max_width {
            area = area.max_width(w);
        }

        if let Some(h) = self.min_scrolled_height {
            area = area.min_scrolled_height(h);
        }

        if let Some(w) = self.min_scrolled_width {
            area = area.min_scrolled_width(w);
        }

        area = area.auto_shrink(self.auto_shrink);
        area = area.scroll_bar_visibility(self.scroll_bar_visibility);
        area = area.animated(self.animated);
        area = area.scroll_source(if self.enable_scrolling {
            ScrollSource::ALL
        } else {
            ScrollSource::NONE
        });

        if let Some(offset) = self.scroll_offset {
            area = area.vertical_scroll_offset(offset.y);
            area = area.horizontal_scroll_offset(offset.x);
        }

        area
    }
}
