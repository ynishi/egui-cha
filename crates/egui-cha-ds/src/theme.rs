//! Theme system for consistent styling
//!
//! Provides a centralized theme system with:
//! - Design tokens (colors, spacing, radii)
//! - `Theme::current()` for component access
//! - `ThemeProvider` trait for external theme integration
//! - TOML-based theme configuration (with `serde` feature)
//!
//! # TOML Configuration Example
//!
//! ```toml
//! base = "dark"
//! primary = "#8B5CF6"
//! spacing_scale = 0.85
//! shadow_blur = 4.0
//! ```
//!
//! # Usage
//!
//! ```ignore
//! // Load theme from TOML file
//! let theme = Theme::load_toml("my-theme.toml")?;
//! theme.apply(&ctx);
//!
//! // Or create from ThemeConfig
//! let config = ThemeConfig {
//!     base: Some("dark".into()),
//!     primary: Some("#8B5CF6".into()),
//!     ..Default::default()
//! };
//! let theme = Theme::from_config(&config);
//! ```

use egui::{Color32, FontId, Id, TextStyle};

/// Theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
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

    // Glass / Transparency (for vibrancy windows)
    /// Glass frame opacity (0.0 = fully transparent, 1.0 = opaque). Default: 0.6
    pub glass_opacity: f32,
    /// Glass frame blur radius (visual hint, actual blur requires vibrancy). Default: 8.0
    pub glass_blur_radius: f32,
    /// Glass frame tint color. None = use bg_primary
    pub glass_tint: Option<Color32>,
    /// Show border on glass frames. Default: true
    pub glass_border: bool,
    /// Custom titlebar height. Default: 32.0
    pub titlebar_height: f32,
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

            // Glass / Transparency
            glass_opacity: 0.6,
            glass_blur_radius: 8.0,
            glass_tint: None, // Use bg_primary
            glass_border: true,
            titlebar_height: 32.0,
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

            // Glass / Transparency (slightly more opaque for dark theme)
            glass_opacity: 0.7,
            glass_blur_radius: 8.0,
            glass_tint: None, // Use bg_primary
            glass_border: true,
            titlebar_height: 32.0,
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

    /// Apply only color-related settings without affecting layout/spacing
    ///
    /// Use this for theme switching (dark/light toggle) to avoid layout changes.
    /// Unlike `apply()`, this only updates Visuals colors, not typography or spacing.
    pub fn apply_colors_only(&self, ctx: &egui::Context) {
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

        // Window stroke color (not width)
        visuals.window_stroke.color = self.border;

        ctx.set_style(style);
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

            // Glass / Transparency (soft for pastel)
            glass_opacity: 0.55,
            glass_blur_radius: 10.0,
            glass_tint: None,
            glass_border: true,
            titlebar_height: 32.0,
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

            // Glass / Transparency (slightly more opaque for pastel dark)
            glass_opacity: 0.65,
            glass_blur_radius: 10.0,
            glass_tint: None,
            glass_border: true,
            titlebar_height: 32.0,
        }
    }
}

// ============================================================================
// TOML Configuration Support (feature = "serde")
// ============================================================================

/// TOML-friendly theme configuration with human-readable color format.
///
/// All fields are optional - unspecified fields inherit from base theme.
///
/// # Color Format
///
/// Colors can be specified as:
/// - Hex: `"#RRGGBB"` or `"#RRGGBBAA"`
/// - RGB: `"rgb(r, g, b)"` or `"rgba(r, g, b, a)"`
///
/// # Example TOML
///
/// ```toml
/// base = "dark"
/// primary = "#8B5CF6"
/// bg_primary = "#1a1a2e"
/// spacing_scale = 0.85
/// font_scale = 1.1
/// shadow_blur = 4.0
/// ```
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Base theme: "light", "dark", "pastel", "pastel_dark"
    pub base: Option<String>,

    // Colors - Primary
    pub primary: Option<String>,
    pub primary_hover: Option<String>,
    pub primary_text: Option<String>,

    // Colors - Secondary
    pub secondary: Option<String>,
    pub secondary_hover: Option<String>,
    pub secondary_text: Option<String>,

    // Colors - Background
    pub bg_primary: Option<String>,
    pub bg_secondary: Option<String>,
    pub bg_tertiary: Option<String>,

    // Colors - Text
    pub text_primary: Option<String>,
    pub text_secondary: Option<String>,
    pub text_muted: Option<String>,

    // Colors - UI State
    pub state_success: Option<String>,
    pub state_warning: Option<String>,
    pub state_danger: Option<String>,
    pub state_info: Option<String>,

    // Colors - Border
    pub border: Option<String>,
    pub border_focus: Option<String>,

    // Scale factors (1.0 = default)
    pub spacing_scale: Option<f32>,
    pub font_scale: Option<f32>,
    pub radius_scale: Option<f32>,
    pub stroke_scale: Option<f32>,

    // Effects
    pub shadow_blur: Option<f32>,
    pub overlay_dim: Option<f32>,
    pub surface_alpha: Option<f32>,

    // Glass / Transparency
    pub glass_opacity: Option<f32>,
    pub glass_blur_radius: Option<f32>,
    pub glass_tint: Option<String>,
    pub glass_border: Option<bool>,
    pub titlebar_height: Option<f32>,
}

#[cfg(feature = "serde")]
impl ThemeConfig {
    /// Parse a color string to Color32.
    ///
    /// Supports:
    /// - `#RRGGBB`
    /// - `#RRGGBBAA`
    /// - `rgb(r, g, b)`
    /// - `rgba(r, g, b, a)`
    pub fn parse_color(s: &str) -> Option<Color32> {
        let s = s.trim();

        // Hex format: #RRGGBB or #RRGGBBAA
        if let Some(hex) = s.strip_prefix('#') {
            return match hex.len() {
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(Color32::from_rgb(r, g, b))
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(Color32::from_rgba_unmultiplied(r, g, b, a))
                }
                _ => None,
            };
        }

        // RGB format: rgb(r, g, b)
        if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].trim().parse().ok()?;
                let g: u8 = parts[1].trim().parse().ok()?;
                let b: u8 = parts[2].trim().parse().ok()?;
                return Some(Color32::from_rgb(r, g, b));
            }
        }

        // RGBA format: rgba(r, g, b, a)
        if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 4 {
                let r: u8 = parts[0].trim().parse().ok()?;
                let g: u8 = parts[1].trim().parse().ok()?;
                let b: u8 = parts[2].trim().parse().ok()?;
                // Alpha can be 0-255 or 0.0-1.0
                let a_str = parts[3].trim();
                let a: u8 = if a_str.contains('.') {
                    let f: f32 = a_str.parse().ok()?;
                    (f * 255.0) as u8
                } else {
                    a_str.parse().ok()?
                };
                return Some(Color32::from_rgba_unmultiplied(r, g, b, a));
            }
        }

        None
    }

    /// Convert Color32 to hex string (#RRGGBB or #RRGGBBAA)
    pub fn color_to_hex(color: Color32) -> String {
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }
}

#[cfg(feature = "serde")]
impl Theme {
    /// Create a Theme from ThemeConfig.
    ///
    /// Starts from base theme and applies overrides.
    pub fn from_config(config: &ThemeConfig) -> Self {
        // Start with base theme
        let mut theme = match config.base.as_deref() {
            Some("dark") => Self::dark(),
            Some("pastel") => Self::pastel(),
            Some("pastel_dark") => Self::pastel_dark(),
            _ => Self::light(), // default
        };

        // Apply color overrides
        macro_rules! apply_color {
            ($field:ident) => {
                if let Some(ref s) = config.$field {
                    if let Some(c) = ThemeConfig::parse_color(s) {
                        theme.$field = c;
                    }
                }
            };
        }

        apply_color!(primary);
        apply_color!(primary_hover);
        apply_color!(primary_text);
        apply_color!(secondary);
        apply_color!(secondary_hover);
        apply_color!(secondary_text);
        apply_color!(bg_primary);
        apply_color!(bg_secondary);
        apply_color!(bg_tertiary);
        apply_color!(text_primary);
        apply_color!(text_secondary);
        apply_color!(text_muted);
        apply_color!(state_success);
        apply_color!(state_warning);
        apply_color!(state_danger);
        apply_color!(state_info);
        apply_color!(border);
        apply_color!(border_focus);

        // Apply scale factors
        if let Some(scale) = config.spacing_scale {
            theme = theme.with_spacing_scale(scale);
        }
        if let Some(scale) = config.font_scale {
            theme = theme.with_font_scale(scale);
        }
        if let Some(scale) = config.radius_scale {
            theme = theme.with_radius_scale(scale);
        }
        if let Some(scale) = config.stroke_scale {
            theme = theme.with_stroke_scale(scale);
        }

        // Apply effects
        if let Some(blur) = config.shadow_blur {
            theme.shadow_blur = Some(blur);
        }
        if let Some(dim) = config.overlay_dim {
            theme.overlay_dim = dim;
        }
        if let Some(alpha) = config.surface_alpha {
            theme.surface_alpha = alpha;
        }

        // Apply glass / transparency settings
        if let Some(opacity) = config.glass_opacity {
            theme.glass_opacity = opacity.clamp(0.0, 1.0);
        }
        if let Some(blur) = config.glass_blur_radius {
            theme.glass_blur_radius = blur.max(0.0);
        }
        if let Some(ref tint) = config.glass_tint {
            theme.glass_tint = ThemeConfig::parse_color(tint);
        }
        if let Some(border) = config.glass_border {
            theme.glass_border = border;
        }
        if let Some(height) = config.titlebar_height {
            theme.titlebar_height = height.max(0.0);
        }

        theme
    }

    /// Convert Theme to ThemeConfig for serialization.
    ///
    /// Exports all color values as hex strings.
    pub fn to_config(&self) -> ThemeConfig {
        ThemeConfig {
            base: Some(match self.variant {
                ThemeVariant::Light => "light".into(),
                ThemeVariant::Dark => "dark".into(),
            }),
            primary: Some(ThemeConfig::color_to_hex(self.primary)),
            primary_hover: Some(ThemeConfig::color_to_hex(self.primary_hover)),
            primary_text: Some(ThemeConfig::color_to_hex(self.primary_text)),
            secondary: Some(ThemeConfig::color_to_hex(self.secondary)),
            secondary_hover: Some(ThemeConfig::color_to_hex(self.secondary_hover)),
            secondary_text: Some(ThemeConfig::color_to_hex(self.secondary_text)),
            bg_primary: Some(ThemeConfig::color_to_hex(self.bg_primary)),
            bg_secondary: Some(ThemeConfig::color_to_hex(self.bg_secondary)),
            bg_tertiary: Some(ThemeConfig::color_to_hex(self.bg_tertiary)),
            text_primary: Some(ThemeConfig::color_to_hex(self.text_primary)),
            text_secondary: Some(ThemeConfig::color_to_hex(self.text_secondary)),
            text_muted: Some(ThemeConfig::color_to_hex(self.text_muted)),
            state_success: Some(ThemeConfig::color_to_hex(self.state_success)),
            state_warning: Some(ThemeConfig::color_to_hex(self.state_warning)),
            state_danger: Some(ThemeConfig::color_to_hex(self.state_danger)),
            state_info: Some(ThemeConfig::color_to_hex(self.state_info)),
            border: Some(ThemeConfig::color_to_hex(self.border)),
            border_focus: Some(ThemeConfig::color_to_hex(self.border_focus)),
            spacing_scale: None, // Not stored, applied at creation
            font_scale: None,
            radius_scale: None,
            stroke_scale: None,
            shadow_blur: self.shadow_blur,
            overlay_dim: Some(self.overlay_dim),
            surface_alpha: Some(self.surface_alpha),
            glass_opacity: Some(self.glass_opacity),
            glass_blur_radius: Some(self.glass_blur_radius),
            glass_tint: self.glass_tint.map(ThemeConfig::color_to_hex),
            glass_border: Some(self.glass_border),
            titlebar_height: Some(self.titlebar_height),
        }
    }

    /// Load theme from TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        let config: ThemeConfig = toml::from_str(toml_str)?;
        Ok(Self::from_config(&config))
    }

    /// Serialize theme to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&self.to_config())
    }

    /// Load theme from TOML file.
    pub fn load_toml(path: impl AsRef<std::path::Path>) -> Result<Self, ThemeLoadError> {
        let content = std::fs::read_to_string(path)?;
        let theme = Self::from_toml(&content)?;
        Ok(theme)
    }

    /// Save theme to TOML file.
    pub fn save_toml(&self, path: impl AsRef<std::path::Path>) -> Result<(), ThemeSaveError> {
        let toml_str = self.to_toml()?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }
}

/// Error loading theme from file
#[cfg(feature = "serde")]
#[derive(Debug)]
pub enum ThemeLoadError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

#[cfg(feature = "serde")]
impl std::fmt::Display for ThemeLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

#[cfg(feature = "serde")]
impl std::error::Error for ThemeLoadError {}

#[cfg(feature = "serde")]
impl From<std::io::Error> for ThemeLoadError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(feature = "serde")]
impl From<toml::de::Error> for ThemeLoadError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

/// Error saving theme to file
#[cfg(feature = "serde")]
#[derive(Debug)]
pub enum ThemeSaveError {
    Io(std::io::Error),
    Serialize(toml::ser::Error),
}

#[cfg(feature = "serde")]
impl std::fmt::Display for ThemeSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Serialize(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

#[cfg(feature = "serde")]
impl std::error::Error for ThemeSaveError {}

#[cfg(feature = "serde")]
impl From<std::io::Error> for ThemeSaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(feature = "serde")]
impl From<toml::ser::Error> for ThemeSaveError {
    fn from(e: toml::ser::Error) -> Self {
        Self::Serialize(e)
    }
}

// ============================================================================
// Lightweight Theme Trait
// ============================================================================

/// Minimal theme trait for quick theme creation.
///
/// Implement only 3 methods to create a basic theme.
/// All other values inherit from the base theme.
///
/// # Example
///
/// ```
/// use egui::Color32;
/// use egui_cha_ds::theme::{LightweightTheme, Theme};
///
/// struct MyBrandTheme;
///
/// impl LightweightTheme for MyBrandTheme {
///     fn primary(&self) -> Color32 {
///         Color32::from_rgb(139, 92, 246)  // Violet
///     }
///     fn background(&self) -> Color32 {
///         Color32::from_rgb(15, 15, 25)  // Dark
///     }
///     fn text(&self) -> Color32 {
///         Color32::from_rgb(240, 240, 250)  // Light
///     }
/// }
///
/// let theme = MyBrandTheme.to_theme();
/// ```
pub trait LightweightTheme {
    /// Primary accent color (buttons, links, highlights)
    fn primary(&self) -> Color32;

    /// Main background color
    fn background(&self) -> Color32;

    /// Primary text color
    fn text(&self) -> Color32;

    /// Convert to full Theme with sensible defaults.
    ///
    /// Automatically derives:
    /// - Hover states (slightly darker/lighter)
    /// - Secondary colors (muted primary)
    /// - Border colors (based on background)
    /// - Text on primary (contrast with primary)
    fn to_theme(&self) -> Theme {
        let primary = self.primary();
        let bg = self.background();
        let text = self.text();

        // Detect if dark or light theme based on background luminance
        let [r, g, b, _] = bg.to_array();
        let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        let is_dark = luminance < 128.0;

        // Start with base theme
        let mut theme = if is_dark {
            Theme::dark()
        } else {
            Theme::light()
        };

        // Apply primary color
        theme.primary = primary;
        theme.primary_hover = if is_dark {
            lighten(primary, 0.15)
        } else {
            darken(primary, 0.15)
        };
        theme.primary_text = contrast_text(primary);
        theme.border_focus = primary;

        // Apply background
        theme.bg_primary = bg;
        theme.bg_secondary = if is_dark {
            lighten(bg, 0.05)
        } else {
            darken(bg, 0.02)
        };
        theme.bg_tertiary = if is_dark {
            lighten(bg, 0.10)
        } else {
            darken(bg, 0.05)
        };

        // Apply text
        theme.text_primary = text;
        theme.text_secondary = with_alpha(text, 0.7);
        theme.text_muted = with_alpha(text, 0.5);

        // Border based on background
        theme.border = if is_dark {
            lighten(bg, 0.15)
        } else {
            darken(bg, 0.10)
        };

        theme
    }
}

// Helper functions for color manipulation

fn lighten(color: Color32, amount: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    let f = 1.0 + amount;
    Color32::from_rgba_unmultiplied(
        ((r as f32 * f).min(255.0)) as u8,
        ((g as f32 * f).min(255.0)) as u8,
        ((b as f32 * f).min(255.0)) as u8,
        a,
    )
}

fn darken(color: Color32, amount: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();
    let f = 1.0 - amount;
    Color32::from_rgba_unmultiplied(
        (r as f32 * f) as u8,
        (g as f32 * f) as u8,
        (b as f32 * f) as u8,
        a,
    )
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let [r, g, b, _] = color.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, (alpha * 255.0) as u8)
}

fn contrast_text(bg: Color32) -> Color32 {
    let [r, g, b, _] = bg.to_array();
    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if luminance > 128.0 {
        Color32::from_rgb(17, 24, 39) // Dark text
    } else {
        Color32::WHITE // Light text
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(
            ThemeConfig::parse_color("#FF0000"),
            Some(Color32::from_rgb(255, 0, 0))
        );
        assert_eq!(
            ThemeConfig::parse_color("#00FF00"),
            Some(Color32::from_rgb(0, 255, 0))
        );
        assert_eq!(
            ThemeConfig::parse_color("#0000FF"),
            Some(Color32::from_rgb(0, 0, 255))
        );
        assert_eq!(
            ThemeConfig::parse_color("#FF000080"),
            Some(Color32::from_rgba_unmultiplied(255, 0, 0, 128))
        );
    }

    #[test]
    fn test_parse_rgb_color() {
        assert_eq!(
            ThemeConfig::parse_color("rgb(255, 0, 0)"),
            Some(Color32::from_rgb(255, 0, 0))
        );
        assert_eq!(
            ThemeConfig::parse_color("rgba(255, 0, 0, 128)"),
            Some(Color32::from_rgba_unmultiplied(255, 0, 0, 128))
        );
        assert_eq!(
            ThemeConfig::parse_color("rgba(255, 0, 0, 0.5)"),
            Some(Color32::from_rgba_unmultiplied(255, 0, 0, 127))
        );
    }

    #[test]
    fn test_color_to_hex() {
        assert_eq!(
            ThemeConfig::color_to_hex(Color32::from_rgb(255, 0, 0)),
            "#FF0000"
        );
        assert_eq!(
            ThemeConfig::color_to_hex(Color32::from_rgba_unmultiplied(255, 0, 0, 128)),
            "#FF000080"
        );
    }

    #[test]
    fn test_theme_from_toml() {
        let toml = r##"
            base = "dark"
            primary = "#8B5CF6"
            spacing_scale = 0.85
        "##;

        let theme = Theme::from_toml(toml).unwrap();
        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.primary, Color32::from_rgb(139, 92, 246));
    }

    #[test]
    fn test_theme_roundtrip() {
        let original = Theme::dark();
        let toml = original.to_toml().unwrap();
        let restored = Theme::from_toml(&toml).unwrap();

        assert_eq!(original.variant, restored.variant);
        assert_eq!(original.primary, restored.primary);
        assert_eq!(original.bg_primary, restored.bg_primary);
    }

    #[test]
    fn test_lightweight_theme() {
        struct TestTheme;
        impl LightweightTheme for TestTheme {
            fn primary(&self) -> Color32 {
                Color32::from_rgb(139, 92, 246)
            }
            fn background(&self) -> Color32 {
                Color32::from_rgb(15, 15, 25)
            }
            fn text(&self) -> Color32 {
                Color32::WHITE
            }
        }

        let theme = TestTheme.to_theme();
        assert_eq!(theme.primary, Color32::from_rgb(139, 92, 246));
        assert_eq!(theme.bg_primary, Color32::from_rgb(15, 15, 25));
        assert_eq!(theme.text_primary, Color32::WHITE);
        assert_eq!(theme.variant, ThemeVariant::Dark); // Detected from bg
    }
}
