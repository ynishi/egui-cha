//! MediaBrowser atom - Thumbnail grid media selector for VJ applications
//!
//! A component for browsing and selecting media files with thumbnail previews.
//! Supports images, videos, and other media types with filtering and search.

use crate::Theme;
use egui::{Color32, Pos2, Rect, Sense, Stroke, TextureId, Ui, Vec2};
use egui_cha::ViewCtx;

/// Media type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MediaType {
    #[default]
    Image,
    Video,
    Audio,
    Text,
    Other,
}

impl MediaType {
    pub fn icon(&self) -> &'static str {
        match self {
            MediaType::Image => "🖼",
            MediaType::Video => "🎬",
            MediaType::Audio => "🎵",
            MediaType::Text => "📄",
            MediaType::Other => "📁",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MediaType::Image => "Image",
            MediaType::Video => "Video",
            MediaType::Audio => "Audio",
            MediaType::Text => "Text",
            MediaType::Other => "Other",
        }
    }
}

/// A media item in the browser
#[derive(Debug, Clone)]
pub struct MediaItem {
    pub id: String,
    pub name: String,
    pub media_type: MediaType,
    pub thumbnail: Option<TextureId>,
    pub duration: Option<f32>,
    pub tags: Vec<String>,
}

impl MediaItem {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            media_type: MediaType::Image,
            thumbnail: None,
            duration: None,
            tags: Vec::new(),
        }
    }

    pub fn with_type(mut self, media_type: MediaType) -> Self {
        self.media_type = media_type;
        self
    }

    pub fn with_thumbnail(mut self, texture: TextureId) -> Self {
        self.thumbnail = Some(texture);
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Browser view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowserViewMode {
    #[default]
    Grid,
    List,
    Compact,
}

/// Events emitted by MediaBrowser
#[derive(Debug, Clone)]
pub enum MediaBrowserEvent {
    Select(String),
    DoubleClick(String),
    ContextMenu(String),
    FilterChange(Option<MediaType>),
    SearchChange(String),
    ViewModeChange(BrowserViewMode),
}

/// Media browser widget
pub struct MediaBrowser<'a> {
    items: &'a [MediaItem],
    selected: Option<&'a str>,
    filter: Option<MediaType>,
    search: &'a str,
    view_mode: BrowserViewMode,
    size: Vec2,
    thumbnail_size: f32,
    columns: Option<usize>,
    show_toolbar: bool,
    show_names: bool,
}

impl<'a> MediaBrowser<'a> {
    pub fn new(items: &'a [MediaItem]) -> Self {
        Self {
            items,
            selected: None,
            filter: None,
            search: "",
            view_mode: BrowserViewMode::Grid,
            size: Vec2::new(400.0, 300.0),
            thumbnail_size: 80.0,
            columns: None,
            show_toolbar: true,
            show_names: true,
        }
    }

    pub fn selected(mut self, id: Option<&'a str>) -> Self {
        self.selected = id;
        self
    }

    pub fn filter(mut self, filter: Option<MediaType>) -> Self {
        self.filter = filter;
        self
    }

    pub fn search(mut self, search: &'a str) -> Self {
        self.search = search;
        self
    }

    pub fn view_mode(mut self, mode: BrowserViewMode) -> Self {
        self.view_mode = mode;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn thumbnail_size(mut self, size: f32) -> Self {
        self.thumbnail_size = size;
        self
    }

    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = Some(cols);
        self
    }

    pub fn show_toolbar(mut self, show: bool) -> Self {
        self.show_toolbar = show;
        self
    }

    pub fn show_names(mut self, show: bool) -> Self {
        self.show_names = show;
        self
    }

    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(MediaBrowserEvent) -> Msg,
    ) {
        if let Some(e) = self.show_internal(ctx.ui) {
            ctx.emit(on_event(e));
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<MediaBrowserEvent> {
        self.show_internal(ui)
    }

    fn show_internal(self, ui: &mut Ui) -> Option<MediaBrowserEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event: Option<MediaBrowserEvent> = None;

        let toolbar_height = if self.show_toolbar { theme.spacing_xl } else { 0.0 };
        let total_height = self.size.y + toolbar_height;

        let (rect, _response) = ui.allocate_exact_size(
            Vec2::new(self.size.x, total_height),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        // Filter items
        let filtered_items: Vec<&MediaItem> = self.items.iter()
            .filter(|item| {
                if let Some(filter) = self.filter {
                    if item.media_type != filter {
                        return false;
                    }
                }
                if !self.search.is_empty() {
                    let search_lower = self.search.to_lowercase();
                    if !item.name.to_lowercase().contains(&search_lower) &&
                       !item.tags.iter().any(|t| t.to_lowercase().contains(&search_lower)) {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Toolbar interactions
        let mut toolbar_filter_clicked: Option<Option<MediaType>> = None;
        let mut view_mode_clicked: Option<BrowserViewMode> = None;

        if self.show_toolbar {
            let toolbar_rect = Rect::from_min_size(rect.min, Vec2::new(self.size.x, toolbar_height));

            // Filter buttons
            let filter_types = [None, Some(MediaType::Image), Some(MediaType::Video), Some(MediaType::Audio)];
            let btn_width = 50.0;
            let mut x = toolbar_rect.min.x + theme.spacing_xs;

            for filter_opt in filter_types.iter() {
                let btn_rect = Rect::from_min_size(
                    Pos2::new(x, toolbar_rect.min.y + theme.spacing_xs),
                    Vec2::new(btn_width, toolbar_height - theme.spacing_sm),
                );
                let resp = ui.allocate_rect(btn_rect, Sense::click());
                if resp.clicked() {
                    toolbar_filter_clicked = Some(*filter_opt);
                }
                x += btn_width + theme.spacing_xs;
            }

            // View mode buttons
            let modes = [BrowserViewMode::Grid, BrowserViewMode::List, BrowserViewMode::Compact];
            let mode_x = toolbar_rect.max.x - (30.0 * 3.0 + theme.spacing_xs * 2.0 + theme.spacing_sm);

            for (i, mode) in modes.iter().enumerate() {
                let mode_rect = Rect::from_min_size(
                    Pos2::new(mode_x + i as f32 * (30.0 + theme.spacing_xs), toolbar_rect.min.y + theme.spacing_xs),
                    Vec2::new(30.0, toolbar_height - theme.spacing_sm),
                );
                let resp = ui.allocate_rect(mode_rect, Sense::click());
                if resp.clicked() {
                    view_mode_clicked = Some(*mode);
                }
            }
        }

        // Content area
        let content_rect = if self.show_toolbar {
            Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + toolbar_height),
                self.size,
            )
        } else {
            Rect::from_min_size(rect.min, self.size)
        };

        // Item interactions
        struct ItemInfo {
            id: String,
            rect: Rect,
            clicked: bool,
            double_clicked: bool,
            secondary_clicked: bool,
            hovered: bool,
        }

        let mut item_infos: Vec<ItemInfo> = Vec::new();

        let (item_width, item_height, cols) = match self.view_mode {
            BrowserViewMode::Grid => {
                let name_height = if self.show_names { theme.spacing_md } else { 0.0 };
                let cols = self.columns.unwrap_or_else(|| {
                    ((content_rect.width() - theme.spacing_xs) / (self.thumbnail_size + theme.spacing_xs)) as usize
                }.max(1));
                (self.thumbnail_size, self.thumbnail_size + name_height, cols)
            }
            BrowserViewMode::List => {
                (content_rect.width() - theme.spacing_sm * 2.0, theme.spacing_xl, 1)
            }
            BrowserViewMode::Compact => {
                let cols = self.columns.unwrap_or(4).max(1);
                (content_rect.width() / cols as f32 - theme.spacing_xs, theme.spacing_lg, cols)
            }
        };

        let spacing = theme.spacing_xs;
        let start_x = content_rect.min.x + spacing;
        let start_y = content_rect.min.y + spacing;

        for (i, item) in filtered_items.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = start_x + col as f32 * (item_width + spacing);
            let y = start_y + row as f32 * (item_height + spacing);

            if y > content_rect.max.y {
                break;
            }

            let item_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(item_width, item_height));
            let resp = ui.allocate_rect(item_rect, Sense::click());

            item_infos.push(ItemInfo {
                id: item.id.clone(),
                rect: item_rect,
                clicked: resp.clicked(),
                double_clicked: resp.double_clicked(),
                secondary_clicked: resp.secondary_clicked(),
                hovered: resp.hovered(),
            });
        }

        // Drawing
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, theme.radius_sm, theme.bg_secondary);

        // Toolbar
        if self.show_toolbar {
            let toolbar_rect = Rect::from_min_size(rect.min, Vec2::new(self.size.x, toolbar_height));
            painter.rect_filled(toolbar_rect, 0.0, theme.bg_tertiary);

            let filter_types: [(Option<MediaType>, &str); 4] = [
                (None, "All"),
                (Some(MediaType::Image), "🖼"),
                (Some(MediaType::Video), "🎬"),
                (Some(MediaType::Audio), "🎵"),
            ];
            let btn_width = 50.0;
            let mut x = toolbar_rect.min.x + theme.spacing_xs;

            for (filter_opt, label) in filter_types.iter() {
                let btn_rect = Rect::from_min_size(
                    Pos2::new(x, toolbar_rect.min.y + theme.spacing_xs),
                    Vec2::new(btn_width, toolbar_height - theme.spacing_sm),
                );
                let is_active = self.filter == *filter_opt;
                let bg = if is_active { theme.primary } else { theme.bg_secondary };
                let text_color = if is_active { theme.primary_text } else { theme.text_secondary };

                painter.rect_filled(btn_rect, theme.radius_sm, bg);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    egui::FontId::proportional(theme.font_size_sm),
                    text_color,
                );
                x += btn_width + theme.spacing_xs;
            }

            // View mode buttons
            let modes: [(BrowserViewMode, &str); 3] = [
                (BrowserViewMode::Grid, "⊞"),
                (BrowserViewMode::List, "☰"),
                (BrowserViewMode::Compact, "⊟"),
            ];
            let mode_x = toolbar_rect.max.x - (30.0 * 3.0 + theme.spacing_xs * 2.0 + theme.spacing_sm);

            for (i, (mode, icon)) in modes.iter().enumerate() {
                let mode_rect = Rect::from_min_size(
                    Pos2::new(mode_x + i as f32 * (30.0 + theme.spacing_xs), toolbar_rect.min.y + theme.spacing_xs),
                    Vec2::new(30.0, toolbar_height - theme.spacing_sm),
                );
                let is_active = self.view_mode == *mode;
                let bg = if is_active { theme.primary } else { theme.bg_secondary };
                let text_color = if is_active { theme.primary_text } else { theme.text_secondary };

                painter.rect_filled(mode_rect, theme.radius_sm, bg);
                painter.text(
                    mode_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *icon,
                    egui::FontId::proportional(theme.font_size_sm),
                    text_color,
                );
            }

            // Item count
            let count_text = format!("{} items", filtered_items.len());
            painter.text(
                Pos2::new(x + theme.spacing_md, toolbar_rect.center().y),
                egui::Align2::LEFT_CENTER,
                &count_text,
                egui::FontId::proportional(theme.font_size_xs),
                theme.text_muted,
            );
        }

        // Content background
        painter.rect_filled(content_rect, 0.0, theme.bg_primary);

        // Items
        for (info, item) in item_infos.iter().zip(filtered_items.iter()) {
            let is_selected = self.selected == Some(&item.id);
            let is_hovered = info.hovered;

            let bg_color = if is_selected {
                theme.primary.gamma_multiply(0.3)
            } else if is_hovered {
                theme.bg_tertiary
            } else {
                Color32::TRANSPARENT
            };

            painter.rect_filled(info.rect, theme.radius_sm, bg_color);

            match self.view_mode {
                BrowserViewMode::Grid => {
                    let thumb_rect = Rect::from_min_size(
                        info.rect.min,
                        Vec2::splat(self.thumbnail_size),
                    );

                    // Thumbnail or placeholder
                    if let Some(tex) = item.thumbnail {
                        painter.image(
                            tex,
                            thumb_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    } else {
                        painter.rect_filled(thumb_rect, theme.radius_sm, theme.bg_tertiary);
                        painter.text(
                            thumb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            item.media_type.icon(),
                            egui::FontId::proportional(self.thumbnail_size * 0.4),
                            theme.text_muted,
                        );
                    }

                    // Duration badge for video/audio
                    if let Some(dur) = item.duration {
                        let dur_text = format_duration(dur);
                        let badge_rect = Rect::from_min_size(
                            Pos2::new(thumb_rect.max.x - 40.0, thumb_rect.max.y - theme.spacing_md),
                            Vec2::new(38.0, theme.spacing_md - 2.0),
                        );
                        painter.rect_filled(badge_rect, 2.0, Color32::from_black_alpha(180));
                        painter.text(
                            badge_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &dur_text,
                            egui::FontId::proportional(theme.font_size_xs * 0.9),
                            Color32::WHITE,
                        );
                    }

                    // Name
                    if self.show_names {
                        let name_y = thumb_rect.max.y + 2.0;
                        let name = truncate_text(&item.name, 12);
                        painter.text(
                            Pos2::new(info.rect.center().x, name_y),
                            egui::Align2::CENTER_TOP,
                            &name,
                            egui::FontId::proportional(theme.font_size_xs),
                            theme.text_primary,
                        );
                    }
                }
                BrowserViewMode::List => {
                    // Icon
                    let icon_rect = Rect::from_min_size(
                        Pos2::new(info.rect.min.x + theme.spacing_xs, info.rect.min.y + theme.spacing_xs),
                        Vec2::splat(info.rect.height() - theme.spacing_sm),
                    );
                    painter.text(
                        icon_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        item.media_type.icon(),
                        egui::FontId::proportional(theme.font_size_md),
                        theme.text_secondary,
                    );

                    // Name
                    painter.text(
                        Pos2::new(icon_rect.max.x + theme.spacing_sm, info.rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &item.name,
                        egui::FontId::proportional(theme.font_size_sm),
                        theme.text_primary,
                    );

                    // Duration
                    if let Some(dur) = item.duration {
                        painter.text(
                            Pos2::new(info.rect.max.x - theme.spacing_sm, info.rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            &format_duration(dur),
                            egui::FontId::proportional(theme.font_size_xs),
                            theme.text_muted,
                        );
                    }
                }
                BrowserViewMode::Compact => {
                    // Icon + name in compact form
                    painter.text(
                        Pos2::new(info.rect.min.x + theme.spacing_xs, info.rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        item.media_type.icon(),
                        egui::FontId::proportional(theme.font_size_sm),
                        theme.text_secondary,
                    );

                    let name = truncate_text(&item.name, 8);
                    painter.text(
                        Pos2::new(info.rect.min.x + theme.spacing_lg, info.rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &name,
                        egui::FontId::proportional(theme.font_size_xs),
                        theme.text_primary,
                    );
                }
            }

            // Selection border
            if is_selected {
                painter.rect_stroke(
                    info.rect,
                    theme.radius_sm,
                    Stroke::new(2.0, theme.primary),
                    egui::StrokeKind::Inside,
                );
            }
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        // Process events
        for info in item_infos.iter() {
            if info.double_clicked {
                event = Some(MediaBrowserEvent::DoubleClick(info.id.clone()));
                break;
            }
            if info.clicked {
                event = Some(MediaBrowserEvent::Select(info.id.clone()));
                break;
            }
            if info.secondary_clicked {
                event = Some(MediaBrowserEvent::ContextMenu(info.id.clone()));
                break;
            }
        }

        if event.is_none() {
            if let Some(filter) = toolbar_filter_clicked {
                event = Some(MediaBrowserEvent::FilterChange(filter));
            } else if let Some(mode) = view_mode_clicked {
                event = Some(MediaBrowserEvent::ViewModeChange(mode));
            }
        }

        event
    }
}

fn format_duration(seconds: f32) -> String {
    let mins = (seconds / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    format!("{}:{:02}", mins, secs)
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max_chars - 1).collect::<String>())
    }
}
