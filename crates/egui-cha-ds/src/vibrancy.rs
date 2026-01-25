//! Window vibrancy effects (blur, acrylic, mica)
//!
//! Provides platform-abstracted window blur/transparency effects.
//!
//! # Platform Support
//!
//! | Platform | Effects |
//! |----------|---------|
//! | macOS 10.10+ | Vibrancy (various materials) |
//! | Windows 7/10 | Blur |
//! | Windows 10+ | Acrylic |
//! | Windows 11 | Mica, Tabbed |
//! | Linux | Unsupported (compositor-dependent) |
//!
//! # Usage
//!
//! ```ignore
//! use egui_cha_ds::vibrancy::{apply_vibrancy, VibrancyEffect};
//!
//! // In your eframe app setup
//! let options = eframe::NativeOptions {
//!     viewport: egui::ViewportBuilder::default()
//!         .with_transparent(true),
//!     ..Default::default()
//! };
//!
//! // After window creation, apply vibrancy
//! apply_vibrancy(&window, VibrancyEffect::default());
//! ```
//!
//! # Requirements
//!
//! - Window must be created with `with_transparent(true)`
//! - App must return `TRANSPARENT` from `clear_color()`
//! - For custom titlebar, use `with_decorations(false)`

use egui::Color32;

/// Vibrancy effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VibrancyEffect {
    /// Automatic: Use best available effect for the platform
    #[default]
    Auto,

    /// Simple blur (Windows 7/10, macOS)
    Blur,

    /// Acrylic effect (Windows 10+)
    Acrylic,

    /// Mica effect (Windows 11)
    Mica,

    /// Tabbed Mica (Windows 11)
    MicaTabbed,

    /// macOS-specific vibrancy materials
    #[cfg(target_os = "macos")]
    MacVibrancy(MacVibrancyMaterial),
}

/// macOS vibrancy material types
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacVibrancyMaterial {
    /// Standard titlebar appearance
    Titlebar,
    /// Selection appearance
    Selection,
    /// Menu appearance
    Menu,
    /// Popover appearance
    Popover,
    /// Sidebar appearance
    Sidebar,
    /// Header view appearance
    HeaderView,
    /// Sheet appearance
    Sheet,
    /// Window background appearance
    WindowBackground,
    /// HUD window appearance
    #[default]
    HudWindow,
    /// Full screen UI appearance
    FullScreenUI,
    /// Tooltip appearance
    ToolTip,
    /// Content background appearance
    ContentBackground,
    /// Under window background appearance
    UnderWindowBackground,
    /// Under page background appearance
    UnderPageBackground,
}

/// Vibrancy configuration
#[derive(Debug, Clone)]
pub struct VibrancyConfig {
    /// Effect to apply
    pub effect: VibrancyEffect,

    /// Tint color (Windows blur/acrylic only)
    /// Format: (R, G, B, A) where A controls opacity
    pub tint: Option<(u8, u8, u8, u8)>,

    /// Use dark mode variant (Windows Mica)
    pub dark_mode: Option<bool>,
}

impl Default for VibrancyConfig {
    fn default() -> Self {
        Self {
            effect: VibrancyEffect::Auto,
            tint: None,
            dark_mode: None,
        }
    }
}

impl VibrancyConfig {
    /// Create with specific effect
    pub fn new(effect: VibrancyEffect) -> Self {
        Self {
            effect,
            ..Default::default()
        }
    }

    /// Set tint color (Windows only)
    pub fn with_tint(mut self, color: Color32) -> Self {
        let [r, g, b, a] = color.to_array();
        self.tint = Some((r, g, b, a));
        self
    }

    /// Set dark mode (Windows Mica only)
    pub fn with_dark_mode(mut self, dark: bool) -> Self {
        self.dark_mode = Some(dark);
        self
    }

    /// Create blur config
    pub fn blur() -> Self {
        Self::new(VibrancyEffect::Blur)
    }

    /// Create acrylic config
    pub fn acrylic() -> Self {
        Self::new(VibrancyEffect::Acrylic)
    }

    /// Create mica config
    pub fn mica() -> Self {
        Self::new(VibrancyEffect::Mica)
    }

    /// Create mica config with dark mode
    pub fn mica_dark() -> Self {
        Self::new(VibrancyEffect::Mica).with_dark_mode(true)
    }
}

/// Result of applying vibrancy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VibrancyResult {
    /// Successfully applied
    Applied(VibrancyEffect),
    /// Platform not supported
    Unsupported,
    /// Effect not available on this OS version
    EffectUnavailable,
    /// Failed to apply
    Failed,
}

impl VibrancyResult {
    /// Check if vibrancy was successfully applied
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::Applied(_))
    }
}

/// Apply vibrancy effect to a window
///
/// # Arguments
///
/// * `window` - Raw window handle (from eframe/winit)
/// * `config` - Vibrancy configuration
///
/// # Returns
///
/// `VibrancyResult` indicating success or failure reason
///
/// # Example
///
/// ```ignore
/// use egui_cha_ds::vibrancy::{apply_vibrancy, VibrancyConfig};
///
/// // Apply with default (auto) settings
/// let result = apply_vibrancy(&window, VibrancyConfig::default());
///
/// // Apply with specific effect
/// let result = apply_vibrancy(&window, VibrancyConfig::mica_dark());
/// ```
#[cfg(target_os = "windows")]
pub fn apply_vibrancy<W: raw_window_handle::HasWindowHandle>(
    window: &W,
    config: VibrancyConfig,
) -> VibrancyResult {
    use window_vibrancy::{apply_acrylic, apply_blur, apply_mica, apply_tabbed};

    let Ok(handle) = window.window_handle() else {
        return VibrancyResult::Failed;
    };

    let effect = match config.effect {
        VibrancyEffect::Auto => {
            // Try Mica first (Windows 11), then Acrylic, then Blur
            if apply_mica(&handle, config.dark_mode).is_ok() {
                return VibrancyResult::Applied(VibrancyEffect::Mica);
            }
            if apply_acrylic(&handle, config.tint).is_ok() {
                return VibrancyResult::Applied(VibrancyEffect::Acrylic);
            }
            if apply_blur(&handle, config.tint).is_ok() {
                return VibrancyResult::Applied(VibrancyEffect::Blur);
            }
            return VibrancyResult::EffectUnavailable;
        }
        VibrancyEffect::Blur => {
            if apply_blur(&handle, config.tint).is_ok() {
                VibrancyEffect::Blur
            } else {
                return VibrancyResult::EffectUnavailable;
            }
        }
        VibrancyEffect::Acrylic => {
            if apply_acrylic(&handle, config.tint).is_ok() {
                VibrancyEffect::Acrylic
            } else {
                return VibrancyResult::EffectUnavailable;
            }
        }
        VibrancyEffect::Mica => {
            if apply_mica(&handle, config.dark_mode).is_ok() {
                VibrancyEffect::Mica
            } else {
                return VibrancyResult::EffectUnavailable;
            }
        }
        VibrancyEffect::MicaTabbed => {
            if apply_tabbed(&handle, config.dark_mode).is_ok() {
                VibrancyEffect::MicaTabbed
            } else {
                return VibrancyResult::EffectUnavailable;
            }
        }
        #[cfg(target_os = "macos")]
        VibrancyEffect::MacVibrancy(_) => return VibrancyResult::EffectUnavailable,
    };

    VibrancyResult::Applied(effect)
}

/// Apply vibrancy effect to a window (macOS)
#[cfg(target_os = "macos")]
pub fn apply_vibrancy<W: raw_window_handle::HasWindowHandle>(
    window: &W,
    config: VibrancyConfig,
) -> VibrancyResult {
    use window_vibrancy::{apply_vibrancy as wv_apply, NSVisualEffectMaterial};

    let Ok(handle) = window.window_handle() else {
        return VibrancyResult::Failed;
    };

    let material = match config.effect {
        VibrancyEffect::Auto | VibrancyEffect::Blur => NSVisualEffectMaterial::HudWindow,
        VibrancyEffect::Acrylic => NSVisualEffectMaterial::FullScreenUI,
        VibrancyEffect::Mica | VibrancyEffect::MicaTabbed => NSVisualEffectMaterial::Sidebar,
        VibrancyEffect::MacVibrancy(mat) => match mat {
            MacVibrancyMaterial::Titlebar => NSVisualEffectMaterial::Titlebar,
            MacVibrancyMaterial::Selection => NSVisualEffectMaterial::Selection,
            MacVibrancyMaterial::Menu => NSVisualEffectMaterial::Menu,
            MacVibrancyMaterial::Popover => NSVisualEffectMaterial::Popover,
            MacVibrancyMaterial::Sidebar => NSVisualEffectMaterial::Sidebar,
            MacVibrancyMaterial::HeaderView => NSVisualEffectMaterial::HeaderView,
            MacVibrancyMaterial::Sheet => NSVisualEffectMaterial::Sheet,
            MacVibrancyMaterial::WindowBackground => NSVisualEffectMaterial::WindowBackground,
            MacVibrancyMaterial::HudWindow => NSVisualEffectMaterial::HudWindow,
            MacVibrancyMaterial::FullScreenUI => NSVisualEffectMaterial::FullScreenUI,
            MacVibrancyMaterial::ToolTip => NSVisualEffectMaterial::Tooltip,
            MacVibrancyMaterial::ContentBackground => NSVisualEffectMaterial::ContentBackground,
            MacVibrancyMaterial::UnderWindowBackground => {
                NSVisualEffectMaterial::UnderWindowBackground
            }
            MacVibrancyMaterial::UnderPageBackground => NSVisualEffectMaterial::UnderPageBackground,
        },
    };

    if wv_apply(&handle, material, None, None).is_ok() {
        VibrancyResult::Applied(config.effect)
    } else {
        VibrancyResult::EffectUnavailable
    }
}

/// Apply vibrancy effect to a window (Linux - unsupported)
#[cfg(target_os = "linux")]
pub fn apply_vibrancy<W: raw_window_handle::HasWindowHandle>(
    _window: &W,
    _config: VibrancyConfig,
) -> VibrancyResult {
    // Linux blur is compositor-dependent, not supported via this API
    VibrancyResult::Unsupported
}

/// Apply vibrancy effect to a window (other platforms - unsupported)
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn apply_vibrancy<W: raw_window_handle::HasWindowHandle>(
    _window: &W,
    _config: VibrancyConfig,
) -> VibrancyResult {
    VibrancyResult::Unsupported
}

/// Clear vibrancy effect from a window
#[cfg(target_os = "windows")]
pub fn clear_vibrancy<W: raw_window_handle::HasWindowHandle>(window: &W) -> bool {
    use window_vibrancy::{clear_acrylic, clear_blur, clear_mica, clear_tabbed};

    let Ok(handle) = window.window_handle() else {
        return false;
    };

    // Try clearing all effects
    let _ = clear_blur(&handle);
    let _ = clear_acrylic(&handle);
    let _ = clear_mica(&handle);
    let _ = clear_tabbed(&handle);
    true
}

/// Clear vibrancy effect from a window (macOS)
#[cfg(target_os = "macos")]
pub fn clear_vibrancy<W: raw_window_handle::HasWindowHandle>(window: &W) -> bool {
    use window_vibrancy::clear_vibrancy as wv_clear;

    let Ok(handle) = window.window_handle() else {
        return false;
    };

    wv_clear(&handle).is_ok()
}

/// Clear vibrancy effect from a window (other platforms)
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn clear_vibrancy<W: raw_window_handle::HasWindowHandle>(_window: &W) -> bool {
    true // Nothing to clear
}

/// Check if vibrancy is supported on current platform
pub fn is_vibrancy_supported() -> bool {
    cfg!(any(target_os = "windows", target_os = "macos"))
}

/// Get available vibrancy effects for current platform
pub fn available_effects() -> &'static [VibrancyEffect] {
    #[cfg(target_os = "windows")]
    {
        &[
            VibrancyEffect::Auto,
            VibrancyEffect::Blur,
            VibrancyEffect::Acrylic,
            VibrancyEffect::Mica,
            VibrancyEffect::MicaTabbed,
        ]
    }

    #[cfg(target_os = "macos")]
    {
        &[VibrancyEffect::Auto, VibrancyEffect::Blur]
        // MacVibrancy variants available but need separate handling
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        &[]
    }
}

// ============================================================================
// Transparent/Glass Frame Components
// ============================================================================

/// A frame with glassmorphism-style appearance
///
/// Provides semi-transparent background with optional blur effect simulation.
/// Works best when window vibrancy is enabled.
///
/// # Example
///
/// ```ignore
/// GlassFrame::new()
///     .opacity(0.7)
///     .blur_radius(8.0)
///     .show(ui, |ui| {
///         ui.label("Content on glass");
///     });
/// ```
#[derive(Clone)]
pub struct GlassFrame {
    /// Background opacity (0.0 = fully transparent, 1.0 = opaque)
    opacity: f32,
    /// Simulated blur radius (visual only, actual blur requires vibrancy)
    blur_radius: f32,
    /// Tint color
    tint: Option<Color32>,
    /// Border
    border: bool,
    /// Corner radius
    corner_radius: f32,
    /// Inner margin
    margin: f32,
}

impl Default for GlassFrame {
    fn default() -> Self {
        Self {
            opacity: 0.6,
            blur_radius: 8.0,
            tint: None,
            border: true,
            corner_radius: 8.0,
            margin: 12.0,
        }
    }
}

impl GlassFrame {
    /// Create a new glass frame
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a glass frame from theme settings
    pub fn from_theme(theme: &crate::Theme) -> Self {
        Self {
            opacity: theme.glass_opacity,
            blur_radius: theme.glass_blur_radius,
            tint: theme.glass_tint,
            border: theme.glass_border,
            corner_radius: theme.radius_md,
            margin: theme.spacing_sm,
        }
    }

    /// Set background opacity (0.0 - 1.0)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set simulated blur radius
    pub fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }

    /// Set tint color
    pub fn tint(mut self, color: Color32) -> Self {
        self.tint = Some(color);
        self
    }

    /// Enable/disable border
    pub fn border(mut self, show: bool) -> Self {
        self.border = show;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set inner margin
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    /// Light glass preset
    pub fn light() -> Self {
        Self::new()
            .tint(Color32::from_rgba_unmultiplied(255, 255, 255, 180))
            .opacity(0.7)
    }

    /// Dark glass preset
    pub fn dark() -> Self {
        Self::new()
            .tint(Color32::from_rgba_unmultiplied(30, 30, 40, 200))
            .opacity(0.8)
    }

    /// Frosted glass preset (higher opacity)
    pub fn frosted() -> Self {
        Self::new().opacity(0.85).blur_radius(12.0)
    }

    /// Show the glass frame with content
    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = crate::Theme::current(ui.ctx());

        // Calculate fill color
        let fill = if let Some(tint) = self.tint {
            let [r, g, b, _] = tint.to_array();
            Color32::from_rgba_unmultiplied(r, g, b, (self.opacity * 255.0) as u8)
        } else {
            // Use theme background with opacity
            let [r, g, b, _] = theme.bg_primary.to_array();
            Color32::from_rgba_unmultiplied(r, g, b, (self.opacity * 255.0) as u8)
        };

        // Border color (subtle)
        let stroke = if self.border {
            let border_alpha = (self.opacity * 0.3 * 255.0) as u8;
            egui::Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, border_alpha),
            )
        } else {
            egui::Stroke::NONE
        };

        egui::Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(self.corner_radius)
            .inner_margin(self.margin)
            .show(ui, add_contents)
    }
}

/// Helper to create transparent viewport options for eframe
///
/// # Example
///
/// ```ignore
/// let options = transparent_viewport_options();
/// eframe::run_native("My App", options, Box::new(|cc| {
///     // Apply vibrancy after window creation
///     Ok(Box::new(MyApp::default()))
/// }));
/// ```
pub fn transparent_viewport_builder() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_transparent(true)
        .with_decorations(false) // Usually needed for custom titlebar
}

/// Trait extension for eframe App to support transparent windows
///
/// # Example
///
/// ```ignore
/// impl eframe::App for MyApp {
///     fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
///         TransparentApp::transparent_clear_color()
///     }
/// }
/// ```
pub trait TransparentApp {
    /// Returns transparent clear color for use with vibrancy
    fn transparent_clear_color() -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}

// Blanket implementation for all types
impl<T> TransparentApp for T {}
