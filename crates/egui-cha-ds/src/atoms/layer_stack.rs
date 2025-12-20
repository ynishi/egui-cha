//! LayerStack atom - VJ layer management with blend modes
//!
//! A component for managing visual layers with opacity, visibility,
//! blend modes, and drag-to-reorder functionality.
//!
//! # Features
//! - Layer visibility toggle
//! - Opacity control per layer
//! - Blend mode selection
//! - Drag-to-reorder layers
//! - Solo/Lock functionality
//! - Theme-aware styling
//!
//! # Example
//! ```ignore
//! LayerStack::new(&layers)
//!     .selected(model.selected_layer)
//!     .show_with(ctx, |event| match event {
//!         LayerEvent::Select(idx) => Msg::SelectLayer(idx),
//!         LayerEvent::Reorder { from, to } => Msg::ReorderLayers(from, to),
//!         LayerEvent::SetOpacity(idx, val) => Msg::SetOpacity(idx, val),
//!         LayerEvent::ToggleVisible(idx) => Msg::ToggleVisible(idx),
//!         LayerEvent::SetBlendMode(idx, mode) => Msg::SetBlendMode(idx, mode),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Blend modes for layer compositing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Difference,
    Exclusion,
    ColorDodge,
    ColorBurn,
}

impl BlendMode {
    /// Get short display name
    pub fn short_name(&self) -> &'static str {
        match self {
            BlendMode::Normal => "Norm",
            BlendMode::Add => "Add",
            BlendMode::Multiply => "Mul",
            BlendMode::Screen => "Scr",
            BlendMode::Overlay => "Ovl",
            BlendMode::Difference => "Diff",
            BlendMode::Exclusion => "Excl",
            BlendMode::ColorDodge => "Dodg",
            BlendMode::ColorBurn => "Burn",
        }
    }

    /// Get all blend modes
    pub fn all() -> &'static [BlendMode] {
        &[
            BlendMode::Normal,
            BlendMode::Add,
            BlendMode::Multiply,
            BlendMode::Screen,
            BlendMode::Overlay,
            BlendMode::Difference,
            BlendMode::Exclusion,
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
        ]
    }
}

/// A single layer's data
#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub solo: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub color: Option<Color32>,
    pub thumbnail: Option<egui::TextureId>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visible: true,
            locked: false,
            solo: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            color: None,
            thumbnail: None,
        }
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Events emitted by LayerStack
#[derive(Debug, Clone)]
pub enum LayerEvent {
    /// Layer was selected
    Select(usize),
    /// Layer visibility toggled
    ToggleVisible(usize),
    /// Layer locked state toggled
    ToggleLock(usize),
    /// Layer solo state toggled
    ToggleSolo(usize),
    /// Layer opacity changed
    SetOpacity(usize, f32),
    /// Layer blend mode changed
    SetBlendMode(usize, BlendMode),
    /// Layers reordered (from, to)
    Reorder { from: usize, to: usize },
    /// Add new layer requested
    AddLayer,
    /// Delete layer requested
    DeleteLayer(usize),
    /// Duplicate layer requested
    DuplicateLayer(usize),
}

/// VJ-style layer stack with blend modes and opacity
pub struct LayerStack<'a> {
    layers: &'a [Layer],
    selected: Option<usize>,
    row_height: f32,
    show_thumbnails: bool,
    show_blend_modes: bool,
    show_controls: bool,
    compact: bool,
}

impl<'a> LayerStack<'a> {
    /// Create a new layer stack
    pub fn new(layers: &'a [Layer]) -> Self {
        Self {
            layers,
            selected: None,
            row_height: 40.0,
            show_thumbnails: true,
            show_blend_modes: true,
            show_controls: true,
            compact: false,
        }
    }

    /// Set selected layer index
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    /// Set row height
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Show/hide layer thumbnails
    pub fn show_thumbnails(mut self, show: bool) -> Self {
        self.show_thumbnails = show;
        self
    }

    /// Show/hide blend mode selector
    pub fn show_blend_modes(mut self, show: bool) -> Self {
        self.show_blend_modes = show;
        self
    }

    /// Show/hide layer controls (add/delete buttons)
    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    /// Use compact mode (smaller UI)
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        if compact {
            self.row_height = 28.0;
            self.show_thumbnails = false;
        }
        self
    }

    /// TEA-style: Show layer stack, emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(LayerEvent) -> Msg,
    ) {
        let event = self.show_internal(ctx.ui);
        if let Some(e) = event {
            ctx.emit(on_event(e));
        }
    }

    /// Show layer stack and return event
    pub fn show(self, ui: &mut Ui) -> Option<LayerEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<LayerEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<LayerEvent> = None;

        let row_height = if self.compact {
            theme.spacing_lg + theme.spacing_sm
        } else {
            self.row_height
        };

        // Calculate widths
        let available_width = ui.available_width();
        let thumbnail_width = if self.show_thumbnails { row_height } else { 0.0 };
        let visibility_width = theme.spacing_lg;
        let lock_width = theme.spacing_lg;
        let opacity_width = if self.compact { 40.0 } else { 60.0 };
        let blend_width = if self.show_blend_modes { 45.0 } else { 0.0 };

        // Header with controls
        if self.show_controls {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Layers").size(theme.font_size_sm).color(theme.text_secondary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_btn = ui.add(
                        egui::Button::new("+")
                            .min_size(Vec2::splat(theme.spacing_lg))
                    );
                    if add_btn.clicked() {
                        event = Some(LayerEvent::AddLayer);
                    }
                });
            });
            ui.add_space(theme.spacing_xs);
        }

        // Collect layer info in first pass
        struct LayerInfo {
            idx: usize,
            rect: Rect,
            row_hovered: bool,
            row_clicked: bool,
            vis_rect: Rect,
            vis_hovered: bool,
            vis_clicked: bool,
            lock_rect: Rect,
            lock_hovered: bool,
            lock_clicked: bool,
            blend_rect: Option<Rect>,
            blend_hovered: bool,
            blend_clicked: bool,
            opacity_rect: Rect,
            opacity_hovered: bool,
            opacity_dragged: bool,
            opacity_drag_pos: Option<Pos2>,
        }

        let mut layer_infos: Vec<LayerInfo> = Vec::with_capacity(self.layers.len());

        // First pass: allocate and collect interactions
        for (idx, _layer) in self.layers.iter().enumerate() {
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(available_width, row_height),
                Sense::click_and_drag(),
            );

            if !ui.is_rect_visible(rect) {
                continue;
            }

            let mut x_offset = rect.min.x + theme.spacing_xs;

            // Visibility rect
            let vis_rect = Rect::from_min_size(
                Pos2::new(x_offset, rect.min.y),
                Vec2::new(visibility_width, row_height),
            );
            let vis_response = ui.allocate_rect(vis_rect, Sense::click());
            x_offset += visibility_width;

            // Lock rect
            let lock_rect = Rect::from_min_size(
                Pos2::new(x_offset, rect.min.y),
                Vec2::new(lock_width, row_height),
            );
            let lock_response = ui.allocate_rect(lock_rect, Sense::click());
            x_offset += lock_width;

            if self.show_thumbnails {
                x_offset += thumbnail_width;
            }

            // Blend mode rect
            let blend_rect = if self.show_blend_modes {
                Some(Rect::from_min_size(
                    Pos2::new(rect.max.x - opacity_width - blend_width - theme.spacing_xs, rect.min.y),
                    Vec2::new(blend_width, row_height),
                ))
            } else {
                None
            };
            let blend_response = blend_rect.map(|r| ui.allocate_rect(r, Sense::click()));

            // Opacity rect
            let opacity_rect = Rect::from_min_size(
                Pos2::new(rect.max.x - opacity_width - theme.spacing_xs, rect.min.y + row_height * 0.3),
                Vec2::new(opacity_width, row_height * 0.4),
            );
            let opacity_response = ui.allocate_rect(opacity_rect, Sense::click_and_drag());

            layer_infos.push(LayerInfo {
                idx,
                rect,
                row_hovered: response.hovered(),
                row_clicked: response.clicked(),
                vis_rect,
                vis_hovered: vis_response.hovered(),
                vis_clicked: vis_response.clicked(),
                lock_rect,
                lock_hovered: lock_response.hovered(),
                lock_clicked: lock_response.clicked(),
                blend_rect,
                blend_hovered: blend_response.as_ref().map_or(false, |r| r.hovered()),
                blend_clicked: blend_response.as_ref().map_or(false, |r| r.clicked()),
                opacity_rect,
                opacity_hovered: opacity_response.hovered(),
                opacity_dragged: opacity_response.dragged(),
                opacity_drag_pos: opacity_response.interact_pointer_pos(),
            });
        }

        // Second pass: draw everything
        let painter = ui.painter();

        for (info, layer) in layer_infos.iter().zip(self.layers.iter()) {
            let is_selected = self.selected == Some(info.idx);

            // Background
            let bg_color = if is_selected {
                theme.primary.gamma_multiply(0.2)
            } else if info.row_hovered {
                theme.bg_secondary
            } else {
                theme.bg_primary
            };
            painter.rect_filled(info.rect, theme.radius_sm, bg_color);

            // Border for selected
            if is_selected {
                painter.rect_stroke(
                    info.rect,
                    theme.radius_sm,
                    Stroke::new(theme.border_width, theme.primary),
                    egui::StrokeKind::Inside,
                );
            }

            // Visibility icon
            let vis_color = if layer.visible {
                if info.vis_hovered { theme.primary } else { theme.text_primary }
            } else {
                theme.text_muted
            };
            painter.text(
                info.vis_rect.center(),
                egui::Align2::CENTER_CENTER,
                if layer.visible { "👁" } else { "○" },
                egui::FontId::proportional(theme.font_size_sm),
                vis_color,
            );

            // Lock icon
            let lock_color = if layer.locked {
                theme.state_warning
            } else if info.lock_hovered {
                theme.text_secondary
            } else {
                theme.text_muted
            };
            painter.text(
                info.lock_rect.center(),
                egui::Align2::CENTER_CENTER,
                if layer.locked { "🔒" } else { "·" },
                egui::FontId::proportional(theme.font_size_xs),
                lock_color,
            );

            // Thumbnail
            if self.show_thumbnails {
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(info.lock_rect.max.x + theme.spacing_xs, info.rect.min.y + theme.spacing_xs),
                    Vec2::splat(row_height - theme.spacing_sm),
                );
                let thumb_color = layer.color.unwrap_or(theme.primary).gamma_multiply(0.5);
                painter.rect_filled(thumb_rect, theme.radius_sm, thumb_color);
                painter.rect_stroke(
                    thumb_rect,
                    theme.radius_sm,
                    Stroke::new(0.5, theme.border),
                    egui::StrokeKind::Inside,
                );
            }

            // Layer name
            let name_x = if self.show_thumbnails {
                info.lock_rect.max.x + row_height + theme.spacing_sm
            } else {
                info.lock_rect.max.x + theme.spacing_sm
            };

            let name_color = if layer.visible {
                if is_selected { theme.text_primary } else { theme.text_secondary }
            } else {
                theme.text_muted
            };

            let name_text = if layer.name.len() > 12 && self.compact {
                format!("{}…", &layer.name[..11])
            } else {
                layer.name.clone()
            };

            painter.text(
                Pos2::new(name_x, info.rect.center().y),
                egui::Align2::LEFT_CENTER,
                &name_text,
                egui::FontId::proportional(if self.compact { theme.font_size_xs } else { theme.font_size_sm }),
                name_color,
            );

            // Blend mode
            if let Some(blend_rect) = info.blend_rect {
                let blend_color = if info.blend_hovered {
                    theme.primary
                } else {
                    theme.text_muted
                };
                painter.text(
                    blend_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    layer.blend_mode.short_name(),
                    egui::FontId::proportional(theme.font_size_xs),
                    blend_color,
                );
            }

            // Opacity bar
            painter.rect_filled(info.opacity_rect, theme.radius_sm, theme.bg_tertiary);

            let fill_width = info.opacity_rect.width() * layer.opacity;
            let fill_rect = Rect::from_min_size(
                info.opacity_rect.min,
                Vec2::new(fill_width, info.opacity_rect.height()),
            );
            let fill_color = if info.opacity_hovered || info.opacity_dragged {
                theme.primary
            } else {
                theme.primary.gamma_multiply(0.7)
            };
            painter.rect_filled(fill_rect, theme.radius_sm, fill_color);

            painter.text(
                info.opacity_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{}%", (layer.opacity * 100.0) as u8),
                egui::FontId::proportional(theme.font_size_xs * 0.9),
                theme.text_primary,
            );

            // Separator line
            if info.idx < self.layers.len() - 1 {
                painter.line_segment(
                    [
                        Pos2::new(info.rect.min.x + theme.spacing_sm, info.rect.max.y),
                        Pos2::new(info.rect.max.x - theme.spacing_sm, info.rect.max.y),
                    ],
                    Stroke::new(0.5, theme.border),
                );
            }

            // Handle events
            if event.is_none() {
                if info.row_clicked {
                    event = Some(LayerEvent::Select(info.idx));
                } else if info.vis_clicked {
                    event = Some(LayerEvent::ToggleVisible(info.idx));
                } else if info.lock_clicked {
                    event = Some(LayerEvent::ToggleLock(info.idx));
                } else if info.blend_clicked {
                    let modes = BlendMode::all();
                    let current_idx = modes.iter().position(|&m| m == layer.blend_mode).unwrap_or(0);
                    let next_idx = (current_idx + 1) % modes.len();
                    event = Some(LayerEvent::SetBlendMode(info.idx, modes[next_idx]));
                } else if info.opacity_dragged {
                    if let Some(pos) = info.opacity_drag_pos {
                        let new_opacity = ((pos.x - info.opacity_rect.min.x) / info.opacity_rect.width()).clamp(0.0, 1.0);
                        event = Some(LayerEvent::SetOpacity(info.idx, new_opacity));
                    }
                }
            }
        }

        event
    }
}
