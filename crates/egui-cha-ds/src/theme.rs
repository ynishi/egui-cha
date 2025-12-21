//! Theme system for consistent styling
//!
//! Provides a centralized theme system with:
//! - Design tokens (colors, spacing, radii)
//! - `Theme::current()` for component access
//! - `ThemeProvider` trait for external theme integration

use egui::{Color32, FontId, Id, TextStyle};

/// Theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    #[default]
    Light,
    Dark,
}

/// Trait for converting external theme systems to DS Theme
pub trait ThemeProvider {
    fn to_ds_theme(&self) -> Theme;
}

/// Design system theme containing all style tokens
#[derive(Debug, Clone)]
pub struct Theme {
    pub variant: ThemeVariant,

    // Colors - Primary
    pub primary: Color32,
    pub primary_hover: Color32,
    pub primary_text: Color32,

    // Colors - Secondary
    pub secondary: Color32,
    pub secondary_hover: Color32,
    pub secondary_text: Color32,

    // Colors - Background
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,

    // Colors - Text
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,

    // Colors - UI State (for buttons, badges, alerts)
    pub state_success: Color32,
    pub state_warning: Color32,
    pub state_danger: Color32,
    pub state_info: Color32,

    // Colors - UI State (text on state background)
    pub state_success_text: Color32,
    pub state_warning_text: Color32,
    pub state_danger_text: Color32,
    pub state_info_text: Color32,

    // Colors - UI State (hover states)
    pub state_success_hover: Color32,
    pub state_warning_hover: Color32,
    pub state_danger_hover: Color32,
    pub state_info_hover: Color32,

    // Colors - Log Severity (for log viewers, console output)
    pub log_debug: Color32,
    pub log_info: Color32,
    pub log_warn: Color32,
    pub log_error: Color32,
    pub log_critical: Color32,

    // Colors - Border
    pub border: Color32,
    pub border_focus: Color32,

    // Spacing
    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,

    // Border radius
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,

    // Stroke / Border width
    pub border_width: f32,
    pub stroke_width: f32,

    // Typography - Font sizes
    pub font_size_xs: f32,
    pub font_size_sm: f32,
    pub font_size_md: f32,
    pub font_size_lg: f32,
    pub font_size_xl: f32,
    pub font_size_2xl: f32,
    pub font_size_3xl: f32,

    // Typography - Line height multiplier
    pub line_height: f32,

    // Overlay / Surface
    /// Dim amount for modal backdrop (0.0 = transparent, 1.0 = opaque black)
    pub overlay_dim: f32,
    /// Alpha for floating surfaces like dropdowns (0.0 = transparent, 1.0 = opaque)
    pub surface_alpha: f32,
    /// Shadow blur radius. None = no shadow (lightweight), Some(4.0) = subtle shadow
    pub shadow_blur: Option<f32>,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    /// Light theme
    pub fn light() -> Self {
        Self {
            variant: ThemeVariant::Light,

            // Primary - Blue
            primary: Color32::from_rgb(59, 130, 246),
            primary_hover: Color32::from_rgb(37, 99, 235),
            primary_text: Color32::WHITE,

            // Secondary - Gray
            secondary: Color32::from_rgb(107, 114, 128),
            secondary_hover: Color32::from_rgb(75, 85, 99),
            secondary_text: Color32::WHITE,

            // Background
            bg_primary: Color32::WHITE,
            bg_secondary: Color32::from_rgb(249, 250, 251),
            bg_tertiary: Color32::from_rgb(243, 244, 246),

            // Text
            text_primary: Color32::from_rgb(17, 24, 39),
            text_secondary: Color32::from_rgb(75, 85, 99),
            text_muted: Color32::from_rgb(156, 163, 175),

            // UI State (for buttons, badges, alerts)
            state_success: Color32::from_rgb(34, 197, 94), // green-500
            state_warning: Color32::from_rgb(245, 158, 11), // amber-500
            state_danger: Color32::from_rgb(239, 68, 68),  // red-500
            state_info: Color32::from_rgb(14, 165, 233),   // sky-500

            // UI State text (on state background)
            state_success_text: Color32::WHITE,
            state_warning_text: Color32::WHITE,
            state_danger_text: Color32::WHITE,
            state_info_text: Color32::WHITE,

            // UI State hover
            state_success_hover: Color32::from_rgb(22, 163, 74), // green-600
            state_warning_hover: Color32::from_rgb(217, 119, 6), // amber-600
            state_danger_hover: Color32::from_rgb(220, 38, 38),  // red-600
            state_info_hover: Color32::from_rgb(2, 132, 199),    // sky-600

            // Log Severity (for log viewers, console output)
            log_debug: Color32::from_rgb(156, 163, 175), // gray-400
            log_info: Color32::from_rgb(59, 130, 246),   // blue-500
            log_warn: Color32::from_rgb(245, 158, 11),   // amber-500
            log_error: Color32::from_rgb(239, 68, 68),   // red-500
            log_critical: Color32::from_rgb(190, 24, 93), // pink-700

            // Border
            border: Color32::from_rgb(229, 231, 235),
            border_focus: Color32::from_rgb(59, 130, 246),

            // Spacing (modern, spacious)
            spacing_xs: 6.0,
            spacing_sm: 12.0,
            spacing_md: 20.0,
            spacing_lg: 32.0,
            spacing_xl: 48.0,

            // Radius
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,

            // Stroke / Border width
            border_width: 1.0,
            stroke_width: 1.0,

            // Typography
            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_2xl: 24.0,
            font_size_3xl: 30.0,
            line_height: 1.4,

            // Overlay / Surface
            overlay_dim: 0.5,
            surface_alpha: 1.0,
            shadow_blur: None, // Lightweight: no shadow
        }
    }

    /// Dark theme
    pub fn dark() -> Self {
        Self {
            variant: ThemeVariant::Dark,

            // Primary - Blue
            primary: Color32::from_rgb(96, 165, 250),
            primary_hover: Color32::from_rgb(59, 130, 246),
            primary_text: Color32::from_rgb(17, 24, 39),

            // Secondary - Gray
            secondary: Color32::from_rgb(156, 163, 175),
            secondary_hover: Color32::from_rgb(107, 114, 128),
            secondary_text: Color32::from_rgb(17, 24, 39),

            // Background
            bg_primary: Color32::from_rgb(17, 24, 39),
            bg_secondary: Color32::from_rgb(31, 41, 55),
            bg_tertiary: Color32::from_rgb(55, 65, 81),

            // Text
            text_primary: Color32::from_rgb(249, 250, 251),
            text_secondary: Color32::from_rgb(209, 213, 219),
            text_muted: Color32::from_rgb(156, 163, 175),

            // UI State (for buttons, badges, alerts)
            state_success: Color32::from_rgb(74, 222, 128), // green-400
            state_warning: Color32::from_rgb(251, 191, 36), // amber-400
            state_danger: Color32::from_rgb(248, 113, 113), // red-400
            state_info: Color32::from_rgb(56, 189, 248),    // sky-400

            // UI State text (on state background) - dark text for light bg
            state_success_text: Color32::from_rgb(17, 24, 39),
            state_warning_text: Color32::from_rgb(17, 24, 39),
            state_danger_text: Color32::from_rgb(17, 24, 39),
            state_info_text: Color32::from_rgb(17, 24, 39),

            // UI State hover
            state_success_hover: Color32::from_rgb(34, 197, 94), // green-500
            state_warning_hover: Color32::from_rgb(245, 158, 11), // amber-500
            state_danger_hover: Color32::from_rgb(239, 68, 68),  // red-500
            state_info_hover: Color32::from_rgb(14, 165, 233),   // sky-500

            // Log Severity (for log viewers, console output)
            log_debug: Color32::from_rgb(209, 213, 219), // gray-300
            log_info: Color32::from_rgb(96, 165, 250),   // blue-400
            log_warn: Color32::from_rgb(251, 191, 36),   // amber-400
            log_error: Color32::from_rgb(248, 113, 113), // red-400
            log_critical: Color32::from_rgb(244, 114, 182), // pink-400

            // Border
            border: Color32::from_rgb(55, 65, 81),
            border_focus: Color32::from_rgb(96, 165, 250),

            // Spacing (modern, spacious - same as light)
            spacing_xs: 6.0,
            spacing_sm: 12.0,
            spacing_md: 20.0,
            spacing_lg: 32.0,
            spacing_xl: 48.0,

            // Radius (same as light)
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,

            // Stroke / Border width (same as light)
            border_width: 1.0,
            stroke_width: 1.0,

            // Typography (same as light)
            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_2xl: 24.0,
            font_size_3xl: 30.0,
            line_height: 1.4,

            // Overlay / Surface (darker for dark theme)
            overlay_dim: 0.7,
            surface_alpha: 1.0,
            shadow_blur: None, // Lightweight: no shadow
        }
    }

    /// ID used for storing theme in egui context
    const STORAGE_ID: &'static str = "egui_cha_ds_theme";

    /// Get current theme from egui context (fallback to default if not set)
    pub fn current(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp::<Theme>(Id::new(Self::STORAGE_ID)))
            .unwrap_or_default()
    }

    /// Create theme from external provider
    pub fn from_provider(provider: impl ThemeProvider) -> Self {
        provider.to_ds_theme()
    }

    /// Apply theme to egui context and store for component access
    pub fn apply(&self, ctx: &egui::Context) {
        // Store theme for component access via Theme::current()
        ctx.data_mut(|d| d.insert_temp(Id::new(Self::STORAGE_ID), self.clone()));

        let mut style = (*ctx.style()).clone();
        let visuals = &mut style.visuals;

        // Dark mode flag
        visuals.dark_mode = self.variant == ThemeVariant::Dark;

        // Background colors
        visuals.panel_fill = self.bg_primary;
        visuals.window_fill = self.bg_primary;
        visuals.extreme_bg_color = self.bg_secondary;
        visuals.faint_bg_color = self.bg_secondary;
        visuals.code_bg_color = self.bg_tertiary;

        // Text colors
        visuals.override_text_color = Some(self.text_primary);
        visuals.hyperlink_color = self.primary;
        visuals.warn_fg_color = self.state_warning;
        visuals.error_fg_color = self.state_danger;

        // Widget styles - noninteractive (labels, separators)
        visuals.widgets.noninteractive.bg_fill = self.bg_secondary;
        visuals.widgets.noninteractive.weak_bg_fill = self.bg_secondary;
        visuals.widgets.noninteractive.bg_stroke.color = self.border;
        visuals.widgets.noninteractive.fg_stroke.color = self.text_primary;

        // Widget styles - inactive (buttons at rest)
        visuals.widgets.inactive.bg_fill = self.bg_tertiary;
        visuals.widgets.inactive.weak_bg_fill = self.bg_tertiary;
        visuals.widgets.inactive.bg_stroke.color = self.border;
        visuals.widgets.inactive.fg_stroke.color = self.text_primary;

        // Widget styles - hovered
        visuals.widgets.hovered.bg_fill = self.primary_hover;
        visuals.widgets.hovered.weak_bg_fill = self.primary_hover;
        visuals.widgets.hovered.bg_stroke.color = self.primary;
        visuals.widgets.hovered.fg_stroke.color = self.primary_text;

        // Widget styles - active (being clicked)
        visuals.widgets.active.bg_fill = self.primary;
        visuals.widgets.active.weak_bg_fill = self.primary;
        visuals.widgets.active.bg_stroke.color = self.primary;
        visuals.widgets.active.fg_stroke.color = self.primary_text;

        // Widget styles - open (dropdown open, etc)
        visuals.widgets.open.bg_fill = self.bg_tertiary;
        visuals.widgets.open.weak_bg_fill = self.bg_tertiary;
        visuals.widgets.open.bg_stroke.color = self.primary;
        visuals.widgets.open.fg_stroke.color = self.text_primary;

        // Selection
        visuals.selection.bg_fill = self.primary.linear_multiply(0.3);
        visuals.selection.stroke.color = self.primary;

        // Stroke widths - Apply to all widget states
        visuals.widgets.noninteractive.bg_stroke.width = self.border_width;
        visuals.widgets.noninteractive.fg_stroke.width = self.stroke_width;
        visuals.widgets.inactive.bg_stroke.width = self.border_width;
        visuals.widgets.inactive.fg_stroke.width = self.stroke_width;
        visuals.widgets.hovered.bg_stroke.width = self.border_width;
        visuals.widgets.hovered.fg_stroke.width = self.stroke_width;
        visuals.widgets.active.bg_stroke.width = self.border_width;
        visuals.widgets.active.fg_stroke.width = self.stroke_width;
        visuals.widgets.open.bg_stroke.width = self.border_width;
        visuals.widgets.open.fg_stroke.width = self.stroke_width;
        visuals.selection.stroke.width = self.stroke_width;

        // Window
        visuals.window_stroke.color = self.border;
        visuals.window_stroke.width = self.border_width;

        // Shadow - configurable via shadow_blur
        match self.shadow_blur {
            None => {
                // Lightweight: no shadow
                visuals.window_shadow = egui::Shadow::NONE;
                visuals.popup_shadow = egui::Shadow::NONE;
            }
            Some(blur) => {
                // Subtle fixed shadow
                let alpha = if self.variant == ThemeVariant::Dark {
                    60
                } else {
                    30
                };
                visuals.window_shadow = egui::Shadow {
                    offset: [0, 2],
                    blur: blur as u8,
                    spread: 0,
                    color: Color32::from_black_alpha(alpha),
                };
                visuals.popup_shadow = visuals.window_shadow;
            }
        }

        // Typography - Configure text styles
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(self.font_size_sm));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(self.font_size_md));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(self.font_size_md));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(self.font_size_xl));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(self.font_size_md));

        // Spacing - Apply theme spacing to egui
        style.spacing.item_spacing = egui::vec2(self.spacing_sm, self.spacing_sm);
        style.spacing.window_margin = egui::Margin::same(self.spacing_md as i8);
        style.spacing.button_padding = egui::vec2(self.spacing_sm, self.spacing_xs);
        style.spacing.menu_margin = egui::Margin::same(self.spacing_sm as i8);
        style.spacing.indent = self.spacing_md;
        style.spacing.icon_spacing = self.spacing_xs;
        style.spacing.icon_width = self.spacing_md;

        ctx.set_style(style);
    }

    /// Apply a scale factor to all spacing values
    ///
    /// This scales `spacing_xs`, `spacing_sm`, `spacing_md`, `spacing_lg`, and `spacing_xl`.
    /// Components using these spacing values will automatically respect the scale.
    ///
    /// # Scale Guidelines
    /// - `0.75` - Compact UI, dense layouts
    /// - `1.0` - Default (no scaling)
    /// - `1.25` - Spacious UI, touch-friendly
    /// - `1.5` - Large UI, accessibility
    ///
    /// # Example
    /// ```
    /// use egui_cha_ds::Theme;
    ///
    /// // Compact theme (75% spacing)
    /// let compact = Theme::dark().with_scale(0.75);
    ///
    /// // Spacious theme (125% spacing)
    /// let spacious = Theme::light().with_scale(1.25);
    /// ```
    ///
    /// # Affected Components
    /// All DS components use theme spacing values, including:
    /// - `ListItem` height (Compact/Medium/Large)
    /// - `Button` padding
    /// - `Card` margins
    /// - `Menu` item spacing
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.spacing_xs *= scale;
        self.spacing_sm *= scale;
        self.spacing_md *= scale;
        self.spacing_lg *= scale;
        self.spacing_xl *= scale;
        self
    }

    /// Apply a scale factor to spacing values only
    ///
    /// Same as [`with_scale`](Self::with_scale), provided for explicit naming.
    pub fn with_spacing_scale(mut self, scale: f32) -> Self {
        self.spacing_xs *= scale;
        self.spacing_sm *= scale;
        self.spacing_md *= scale;
        self.spacing_lg *= scale;
        self.spacing_xl *= scale;
        self
    }

    /// Apply a scale factor to border radius values
    ///
    /// Scales `radius_sm`, `radius_md`, `radius_lg` for rounded corners.
    ///
    /// # Example
    /// ```
    /// use egui_cha_ds::Theme;
    ///
    /// // Sharper corners
    /// let sharp = Theme::light().with_radius_scale(0.5);
    ///
    /// // More rounded
    /// let rounded = Theme::dark().with_radius_scale(2.0);
    /// ```
    pub fn with_radius_scale(mut self, scale: f32) -> Self {
        self.radius_sm *= scale;
        self.radius_md *= scale;
        self.radius_lg *= scale;
        self
    }

    /// Apply a scale factor to font sizes
    ///
    /// Scales all font size tokens from `font_size_xs` to `font_size_3xl`.
    /// Useful for accessibility or density preferences.
    ///
    /// # Example
    /// ```
    /// use egui_cha_ds::Theme;
    ///
    /// // Larger text for accessibility
    /// let accessible = Theme::light().with_font_scale(1.2);
    ///
    /// // Smaller text for dense displays
    /// let dense = Theme::dark().with_font_scale(0.9);
    /// ```
    pub fn with_font_scale(mut self, scale: f32) -> Self {
        self.font_size_xs *= scale;
        self.font_size_sm *= scale;
        self.font_size_md *= scale;
        self.font_size_lg *= scale;
        self.font_size_xl *= scale;
        self.font_size_2xl *= scale;
        self.font_size_3xl *= scale;
        self
    }

    /// Apply a scale factor to stroke and border widths
    ///
    /// Scales `border_width` and `stroke_width` for thicker/thinner lines.
    ///
    /// # Example
    /// ```
    /// use egui_cha_ds::Theme;
    ///
    /// // Bolder borders
    /// let bold = Theme::light().with_stroke_scale(2.0);
    /// ```
    pub fn with_stroke_scale(mut self, scale: f32) -> Self {
        self.border_width *= scale;
        self.stroke_width *= scale;
        self
    }

    /// Enable subtle shadow (default: 4.0 blur)
    ///
    /// # Example
    /// ```
    /// use egui_cha_ds::Theme;
    ///
    /// // Enable default subtle shadow
    /// let with_shadow = Theme::light().with_shadow();
    ///
    /// // Custom blur radius
    /// let soft_shadow = Theme::dark().with_shadow_blur(8.0);
    /// ```
    pub fn with_shadow(self) -> Self {
        self.with_shadow_blur(4.0)
    }

    /// Enable shadow with custom blur radius
    pub fn with_shadow_blur(mut self, blur: f32) -> Self {
        self.shadow_blur = Some(blur);
        self
    }

    /// Pastel theme - soft, modern colors
    pub fn pastel() -> Self {
        Self {
            variant: ThemeVariant::Light,

            // Primary - Soft lavender
            primary: Color32::from_rgb(167, 139, 250), // violet-400
            primary_hover: Color32::from_rgb(139, 92, 246), // violet-500
            primary_text: Color32::WHITE,

            // Secondary - Soft pink
            secondary: Color32::from_rgb(244, 114, 182), // pink-400
            secondary_hover: Color32::from_rgb(236, 72, 153), // pink-500
            secondary_text: Color32::WHITE,

            // Background - Cream/off-white
            bg_primary: Color32::from_rgb(255, 251, 245), // warm white
            bg_secondary: Color32::from_rgb(254, 243, 235), // peach-50
            bg_tertiary: Color32::from_rgb(253, 235, 223), // peach-100

            // Text - Soft dark
            text_primary: Color32::from_rgb(64, 57, 72), // muted purple-gray
            text_secondary: Color32::from_rgb(107, 98, 116), // lighter
            text_muted: Color32::from_rgb(156, 148, 163), // even lighter

            // UI State (for buttons, badges, alerts) - Pastel versions
            state_success: Color32::from_rgb(134, 239, 172), // green-300
            state_warning: Color32::from_rgb(253, 224, 71),  // yellow-300
            state_danger: Color32::from_rgb(253, 164, 175),  // rose-300
            state_info: Color32::from_rgb(147, 197, 253),    // blue-300

            // UI State text
            state_success_text: Color32::from_rgb(22, 101, 52), // green-800
            state_warning_text: Color32::from_rgb(133, 77, 14), // amber-800
            state_danger_text: Color32::from_rgb(159, 18, 57),  // rose-800
            state_info_text: Color32::from_rgb(30, 64, 175),    // blue-800

            // UI State hover
            state_success_hover: Color32::from_rgb(74, 222, 128), // green-400
            state_warning_hover: Color32::from_rgb(250, 204, 21), // yellow-400
            state_danger_hover: Color32::from_rgb(251, 113, 133), // rose-400
            state_info_hover: Color32::from_rgb(96, 165, 250),    // blue-400

            // Log Severity (for log viewers, console output)
            log_debug: Color32::from_rgb(156, 148, 163), // muted purple-gray
            log_info: Color32::from_rgb(96, 165, 250),   // blue-400
            log_warn: Color32::from_rgb(250, 204, 21),   // yellow-400
            log_error: Color32::from_rgb(251, 113, 133), // rose-400
            log_critical: Color32::from_rgb(236, 72, 153), // pink-500

            // Border - Soft
            border: Color32::from_rgb(233, 213, 202), // warm gray
            border_focus: Color32::from_rgb(167, 139, 250), // violet-400

            // Spacing (modern, spacious)
            spacing_xs: 6.0,
            spacing_sm: 12.0,
            spacing_md: 20.0,
            spacing_lg: 32.0,
            spacing_xl: 48.0,

            // Radius - More rounded for soft look
            radius_sm: 6.0,
            radius_md: 12.0,
            radius_lg: 16.0,

            // Stroke / Border width
            border_width: 1.0,
            stroke_width: 1.0,

            // Typography (same as light)
            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_2xl: 24.0,
            font_size_3xl: 30.0,
            line_height: 1.4,

            // Overlay / Surface (softer for pastel)
            overlay_dim: 0.4,
            surface_alpha: 1.0,
            shadow_blur: None, // Lightweight: no shadow
        }
    }

    /// Pastel dark theme - soft colors on dark background
    pub fn pastel_dark() -> Self {
        Self {
            variant: ThemeVariant::Dark,

            // Primary - Soft lavender
            primary: Color32::from_rgb(196, 181, 253), // violet-300
            primary_hover: Color32::from_rgb(167, 139, 250), // violet-400
            primary_text: Color32::from_rgb(30, 27, 38), // dark purple

            // Secondary - Soft pink
            secondary: Color32::from_rgb(249, 168, 212), // pink-300
            secondary_hover: Color32::from_rgb(244, 114, 182), // pink-400
            secondary_text: Color32::from_rgb(30, 27, 38),

            // Background - Deep purple-gray
            bg_primary: Color32::from_rgb(24, 22, 32), // deep purple
            bg_secondary: Color32::from_rgb(32, 29, 43), // slightly lighter
            bg_tertiary: Color32::from_rgb(45, 41, 58), // even lighter

            // Text - Soft light
            text_primary: Color32::from_rgb(243, 237, 255), // soft white
            text_secondary: Color32::from_rgb(196, 189, 210),
            text_muted: Color32::from_rgb(140, 133, 156),

            // UI State (for buttons, badges, alerts) - Muted pastel on dark
            state_success: Color32::from_rgb(74, 222, 128), // green-400
            state_warning: Color32::from_rgb(250, 204, 21), // yellow-400
            state_danger: Color32::from_rgb(251, 113, 133), // rose-400
            state_info: Color32::from_rgb(96, 165, 250),    // blue-400

            // UI State text (dark on light bg)
            state_success_text: Color32::from_rgb(20, 30, 25),
            state_warning_text: Color32::from_rgb(35, 30, 15),
            state_danger_text: Color32::from_rgb(35, 20, 25),
            state_info_text: Color32::from_rgb(20, 25, 35),

            // UI State hover
            state_success_hover: Color32::from_rgb(134, 239, 172),
            state_warning_hover: Color32::from_rgb(253, 224, 71),
            state_danger_hover: Color32::from_rgb(253, 164, 175),
            state_info_hover: Color32::from_rgb(147, 197, 253),

            // Log Severity (for log viewers, console output)
            log_debug: Color32::from_rgb(140, 133, 156), // muted
            log_info: Color32::from_rgb(147, 197, 253),  // blue-300
            log_warn: Color32::from_rgb(253, 224, 71),   // yellow-300
            log_error: Color32::from_rgb(253, 164, 175), // rose-300
            log_critical: Color32::from_rgb(249, 168, 212), // pink-300

            // Border
            border: Color32::from_rgb(55, 50, 70),
            border_focus: Color32::from_rgb(196, 181, 253),

            // Spacing (modern, spacious)
            spacing_xs: 6.0,
            spacing_sm: 12.0,
            spacing_md: 20.0,
            spacing_lg: 32.0,
            spacing_xl: 48.0,

            // Radius - More rounded
            radius_sm: 6.0,
            radius_md: 12.0,
            radius_lg: 16.0,

            // Stroke / Border width
            border_width: 1.0,
            stroke_width: 1.0,

            // Typography (same as light)
            font_size_xs: 10.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
            font_size_2xl: 24.0,
            font_size_3xl: 30.0,
            line_height: 1.4,

            // Overlay / Surface (softer for pastel dark)
            overlay_dim: 0.6,
            surface_alpha: 1.0,
            shadow_blur: None, // Lightweight: no shadow
        }
    }
}
