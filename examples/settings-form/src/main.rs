//! Settings Form Example
//!
//! Demonstrates egui-cha layout builders with a settings form.
//!
//! Key insight:
//! - Builder API (col/row/grid) works great for **layout structure**
//! - For complex forms with mutable state, break into smaller units
//! - Each closure should only borrow what it needs

use eframe::egui;
use egui_cha_ds::cha::{col, row, grid};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_title("Settings Form - egui-cha layout demo"),
        ..Default::default()
    };

    eframe::run_native(
        "settings-form",
        options,
        Box::new(|_cc| Ok(Box::new(SettingsApp::default()))),
    )
}

/// User settings (~12 parameters)
#[derive(Default)]
struct UserSettings {
    // Profile
    username: String,
    email: String,
    display_name: String,
    bio: String,

    // Appearance
    dark_mode: bool,
    font_size: f32,
    accent_color: [f32; 3],

    // Notifications
    email_notifications: bool,
    push_notifications: bool,
    notification_sound: bool,

    // Privacy
    profile_public: bool,
    show_online_status: bool,

    // Advanced
    auto_save: bool,
    backup_frequency: usize,
}

struct SettingsApp {
    settings: UserSettings,
    active_tab: usize,
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self {
            settings: UserSettings {
                username: "johndoe".into(),
                email: "john@example.com".into(),
                display_name: "John Doe".into(),
                bio: "Hello, I'm John!".into(),
                dark_mode: true,
                font_size: 14.0,
                accent_color: [0.2, 0.6, 1.0],
                email_notifications: true,
                push_notifications: false,
                notification_sound: true,
                profile_public: true,
                show_online_status: false,
                auto_save: true,
                backup_frequency: 1,
            },
            active_tab: 0,
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            col()
                .spacing(12.0)
                .padding(16.0)
                .add(|ui: &mut egui::Ui| {
                    ui.heading("⚙ Settings");
                })
                .show(ui);

            // Tab selector
            ui.horizontal(|ui| {
                let tabs = ["Profile", "Appearance", "Notifications", "Privacy"];
                for (i, tab) in tabs.iter().enumerate() {
                    if ui.selectable_label(self.active_tab == i, *tab).clicked() {
                        self.active_tab = i;
                    }
                }
            });
            ui.separator();

            // Content
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.active_tab {
                    0 => self.profile_tab(ui),
                    1 => self.appearance_tab(ui),
                    2 => self.notifications_tab(ui),
                    3 => self.privacy_tab(ui),
                    _ => {}
                }
            });
        });
    }
}

impl SettingsApp {
    fn profile_tab(&mut self, ui: &mut egui::Ui) {
        // Demo: Using grid for form layout
        col()
            .spacing(12.0)
            .padding(8.0)
            .add(|ui: &mut egui::Ui| {
                ui.strong("Profile Information");
            })
            .show(ui);

        // Form fields - each in its own egui call to avoid borrow issues
        grid(2).gap(8.0)
            .add(|ui: &mut egui::Ui| { ui.label("Username:"); })
            .add(|ui: &mut egui::Ui| {
                ui.text_edit_singleline(&mut self.settings.username);
            })
            .show(ui);

        grid(2).gap(8.0)
            .add(|ui: &mut egui::Ui| { ui.label("Email:"); })
            .add(|ui: &mut egui::Ui| {
                ui.text_edit_singleline(&mut self.settings.email);
            })
            .show(ui);

        grid(2).gap(8.0)
            .add(|ui: &mut egui::Ui| { ui.label("Display Name:"); })
            .add(|ui: &mut egui::Ui| {
                ui.text_edit_singleline(&mut self.settings.display_name);
            })
            .show(ui);

        grid(2).gap(8.0)
            .add(|ui: &mut egui::Ui| { ui.label("Bio:"); })
            .add(|ui: &mut egui::Ui| {
                ui.text_edit_multiline(&mut self.settings.bio);
            })
            .show(ui);

        ui.add_space(12.0);
        row()
            .spacing(8.0)
            .add(|ui: &mut egui::Ui| {
                ui.button("Save");
            })
            .add(|ui: &mut egui::Ui| {
                ui.button("Cancel");
            })
            .show(ui);
    }

    fn appearance_tab(&mut self, ui: &mut egui::Ui) {
        col()
            .spacing(12.0)
            .padding(8.0)
            .add(|ui: &mut egui::Ui| {
                ui.strong("Appearance Settings");
            })
            .show(ui);

        row().spacing(12.0)
            .add(|ui: &mut egui::Ui| { ui.label("Dark Mode:"); })
            .add(|ui: &mut egui::Ui| {
                ui.checkbox(&mut self.settings.dark_mode, "");
            })
            .show(ui);

        row().spacing(12.0)
            .add(|ui: &mut egui::Ui| { ui.label("Font Size:"); })
            .add(|ui: &mut egui::Ui| {
                ui.add(egui::Slider::new(&mut self.settings.font_size, 10.0..=24.0));
            })
            .show(ui);

        row().spacing(12.0)
            .add(|ui: &mut egui::Ui| { ui.label("Accent Color:"); })
            .add(|ui: &mut egui::Ui| {
                ui.color_edit_button_rgb(&mut self.settings.accent_color);
            })
            .show(ui);
    }

    fn notifications_tab(&mut self, ui: &mut egui::Ui) {
        col()
            .spacing(12.0)
            .padding(8.0)
            .add(|ui: &mut egui::Ui| {
                ui.strong("Notification Preferences");
            })
            .show(ui);

        ui.checkbox(&mut self.settings.email_notifications, "Email Notifications");
        ui.checkbox(&mut self.settings.push_notifications, "Push Notifications");
        ui.checkbox(&mut self.settings.notification_sound, "Notification Sound");
    }

    fn privacy_tab(&mut self, ui: &mut egui::Ui) {
        col()
            .spacing(12.0)
            .padding(8.0)
            .add(|ui: &mut egui::Ui| {
                ui.strong("Privacy Settings");
            })
            .show(ui);

        ui.checkbox(&mut self.settings.profile_public, "Public Profile");
        ui.checkbox(&mut self.settings.show_online_status, "Show Online Status");

        ui.add_space(16.0);

        row()
            .spacing(8.0)
            .add(|ui: &mut egui::Ui| {
                ui.label("Auto Save:");
            })
            .add(|ui: &mut egui::Ui| {
                ui.checkbox(&mut self.settings.auto_save, "");
            })
            .show(ui);
    }
}
