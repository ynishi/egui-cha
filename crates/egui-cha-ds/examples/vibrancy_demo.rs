//! Vibrancy Demo - Transparent window with glass effects
//!
//! Run with: cargo run -p egui-cha-ds --example vibrancy_demo --features vibrancy

use eframe::egui;
use egui_cha_ds::titlebar::{TitleBar, TitleBarButtonStyle, TitleBarStyle};
use egui_cha_ds::vibrancy::{available_effects, is_vibrancy_supported, GlassFrame, VibrancyEffect};
use egui_cha_ds::Theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_transparent(true)
            .with_decorations(false)
            .with_inner_size([600.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibrancy Demo",
        options,
        Box::new(|cc| {
            // Set up fonts
            egui_cha_ds::setup_fonts(&cc.egui_ctx);

            // Apply default theme (dark)
            let theme = Theme::dark();
            theme.apply(&cc.egui_ctx);

            Ok(Box::new(VibrancyDemoApp::new(theme)))
        }),
    )
}

struct VibrancyDemoApp {
    theme: Theme,
    glass_opacity: f32,
    dark_mode: bool,
    titlebar_style: usize,
}

impl VibrancyDemoApp {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            glass_opacity: 0.7,
            dark_mode: true,
            titlebar_style: 0,
        }
    }

    fn effect_name(effect: &VibrancyEffect) -> &'static str {
        match effect {
            VibrancyEffect::Auto => "Auto",
            VibrancyEffect::Blur => "Blur",
            VibrancyEffect::Acrylic => "Acrylic",
            VibrancyEffect::Mica => "Mica",
            VibrancyEffect::MicaTabbed => "Mica Tabbed",
            #[cfg(target_os = "macos")]
            VibrancyEffect::MacVibrancy(_) => "Mac Vibrancy",
        }
    }
}

impl eframe::App for VibrancyDemoApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Transparent background for vibrancy
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Track if theme needs to be updated (handled after UI to avoid borrow conflicts)
        let mut theme_changed = false;

        // Clone theme for use in closures (avoids borrow issues)
        let theme = self.theme.clone();

        // Main panel with transparent background
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                // Titlebar with frosted glass background
                GlassFrame::frosted()
                    .corner_radius(0.0)
                    .margin(0.0)
                    .show(ui, |ui| {
                        let titlebar_style = match self.titlebar_style {
                            0 => TitleBarButtonStyle::TrafficLights,
                            1 => TitleBarButtonStyle::WindowsIcons,
                            _ => TitleBarButtonStyle::Minimal,
                        };

                        let mut style = TitleBarStyle::transparent_from_theme(&theme);
                        style.button_style = titlebar_style;

                        let response = TitleBar::new("Vibrancy Demo")
                            .id("main_titlebar")
                            .style(style)
                            .show(ui);

                        if response.close_clicked {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                ui.add_space(8.0);

                // Content area
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    ui.vertical(|ui| {
                        // Status
                        GlassFrame::from_theme(&theme)
                            .opacity(self.glass_opacity)
                            .show(ui, |ui| {
                                ui.heading("Vibrancy Status");
                                ui.add_space(4.0);

                                ui.horizontal(|ui| {
                                    ui.label("Platform Support:");
                                    if is_vibrancy_supported() {
                                        ui.colored_label(theme.state_success, "Supported");
                                    } else {
                                        ui.colored_label(theme.state_danger, "Not Supported");
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Window:");
                                    ui.colored_label(
                                        theme.text_secondary,
                                        "Transparent (no decorations)",
                                    );
                                });

                                ui.small("Note: Actual blur requires platform compositor.");

                                ui.add_space(8.0);
                                ui.label("Available Effects:");
                                for effect in available_effects() {
                                    ui.label(format!("  • {}", Self::effect_name(effect)));
                                }
                            });

                        ui.add_space(12.0);

                        // Controls
                        GlassFrame::from_theme(&theme)
                            .opacity(self.glass_opacity)
                            .show(ui, |ui| {
                                ui.heading("Controls");
                                ui.add_space(4.0);

                                // Glass opacity slider
                                ui.horizontal(|ui| {
                                    ui.label("Glass Opacity:");
                                    ui.add(egui::Slider::new(&mut self.glass_opacity, 0.0..=1.0));
                                });

                                // Dark mode toggle
                                ui.horizontal(|ui| {
                                    ui.label("Dark Mode:");
                                    if ui.checkbox(&mut self.dark_mode, "").changed() {
                                        theme_changed = true;
                                    }
                                });

                                // Titlebar style
                                ui.horizontal(|ui| {
                                    ui.label("Titlebar Style:");
                                    egui::ComboBox::from_id_salt("titlebar_style")
                                        .selected_text(match self.titlebar_style {
                                            0 => "TrafficLights",
                                            1 => "Windows",
                                            _ => "Minimal",
                                        })
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut self.titlebar_style,
                                                0,
                                                "TrafficLights",
                                            );
                                            ui.selectable_value(
                                                &mut self.titlebar_style,
                                                1,
                                                "Windows",
                                            );
                                            ui.selectable_value(
                                                &mut self.titlebar_style,
                                                2,
                                                "Minimal",
                                            );
                                        });
                                });
                            });

                        ui.add_space(12.0);

                        // Glass presets
                        ui.horizontal(|ui| {
                            GlassFrame::light().show(ui, |ui| {
                                ui.label("Light");
                            });

                            ui.add_space(8.0);

                            GlassFrame::dark().show(ui, |ui| {
                                ui.label("Dark");
                            });

                            ui.add_space(8.0);

                            GlassFrame::frosted().show(ui, |ui| {
                                ui.label("Frosted");
                            });
                        });

                        ui.add_space(12.0);

                        // Sample content
                        GlassFrame::from_theme(&theme)
                            .opacity(self.glass_opacity)
                            .show(ui, |ui| {
                                ui.heading("Sample Content");
                                ui.add_space(4.0);
                                ui.label("This is a glass frame with content.");
                                ui.label("The background should show through.");
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    if ui.button("Button 1").clicked() {}
                                    if ui.button("Button 2").clicked() {}
                                    if ui.button("Button 3").clicked() {}
                                });
                            });

                        ui.add_space(16.0);

                        // Instructions
                        ui.small("Note: Vibrancy effect depends on OS and compositor.");
                        ui.small("macOS: 10.10+, Windows: 10/11");
                    });
                    ui.add_space(16.0);
                });
            });

        // Update theme after UI (avoids borrow conflicts)
        if theme_changed {
            self.theme = if self.dark_mode {
                Theme::dark()
            } else {
                Theme::light()
            };
            // Apply colors only (no typography/spacing changes)
            self.theme.apply_colors_only(ctx);
        }
    }
}
