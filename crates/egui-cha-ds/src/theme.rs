//! Theme system for consistent styling
//!
//! Provides a centralized theme system with:
//! - Design tokens (colors, spacing, radii)
//! - `Theme::current()` for component access
//! - `ThemeProvider` trait for external theme integration

use egui::{Color32, Id};

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

    // Colors - Semantic (background)
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,
    pub danger: Color32,

    // Colors - Semantic (text on semantic background)
    pub success_text: Color32,
    pub warning_text: Color32,
    pub error_text: Color32,
    pub info_text: Color32,
    pub danger_text: Color32,

    // Colors - Semantic (hover states)
    pub success_hover: Color32,
    pub warning_hover: Color32,
    pub error_hover: Color32,
    pub info_hover: Color32,
    pub danger_hover: Color32,

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

            // Semantic (background)
            success: Color32::from_rgb(34, 197, 94),   // green-500
            warning: Color32::from_rgb(245, 158, 11),  // amber-500
            error: Color32::from_rgb(239, 68, 68),     // red-500
            info: Color32::from_rgb(14, 165, 233),     // sky-500
            danger: Color32::from_rgb(239, 68, 68),    // red-500 (alias)

            // Semantic text (on semantic background)
            success_text: Color32::WHITE,
            warning_text: Color32::WHITE,
            error_text: Color32::WHITE,
            info_text: Color32::WHITE,
            danger_text: Color32::WHITE,

            // Semantic hover
            success_hover: Color32::from_rgb(22, 163, 74),   // green-600
            warning_hover: Color32::from_rgb(217, 119, 6),   // amber-600
            error_hover: Color32::from_rgb(220, 38, 38),     // red-600
            info_hover: Color32::from_rgb(2, 132, 199),      // sky-600
            danger_hover: Color32::from_rgb(220, 38, 38),    // red-600

            // Border
            border: Color32::from_rgb(229, 231, 235),
            border_focus: Color32::from_rgb(59, 130, 246),

            // Spacing
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,

            // Radius
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,
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

            // Semantic (background)
            success: Color32::from_rgb(74, 222, 128),   // green-400
            warning: Color32::from_rgb(251, 191, 36),   // amber-400
            error: Color32::from_rgb(248, 113, 113),    // red-400
            info: Color32::from_rgb(56, 189, 248),      // sky-400
            danger: Color32::from_rgb(248, 113, 113),   // red-400 (alias)

            // Semantic text (on semantic background) - dark text for light bg
            success_text: Color32::from_rgb(17, 24, 39),
            warning_text: Color32::from_rgb(17, 24, 39),
            error_text: Color32::from_rgb(17, 24, 39),
            info_text: Color32::from_rgb(17, 24, 39),
            danger_text: Color32::from_rgb(17, 24, 39),

            // Semantic hover
            success_hover: Color32::from_rgb(34, 197, 94),   // green-500
            warning_hover: Color32::from_rgb(245, 158, 11),  // amber-500
            error_hover: Color32::from_rgb(239, 68, 68),     // red-500
            info_hover: Color32::from_rgb(14, 165, 233),     // sky-500
            danger_hover: Color32::from_rgb(239, 68, 68),    // red-500

            // Border
            border: Color32::from_rgb(55, 65, 81),
            border_focus: Color32::from_rgb(96, 165, 250),

            // Spacing (same as light)
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,

            // Radius (same as light)
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,
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
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.error;

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

        // Window
        visuals.window_stroke.color = self.border;
        visuals.window_shadow.color = if self.variant == ThemeVariant::Dark {
            Color32::from_black_alpha(100)
        } else {
            Color32::from_black_alpha(40)
        };

        // Popup
        visuals.popup_shadow.color = visuals.window_shadow.color;

        ctx.set_style(style);
    }
}
