//! Navbar molecule - Navigation bar with router integration

use crate::atoms::icons;
use egui::{Color32, RichText};
use egui_cha::router::{Router, RouterMsg};
use egui_cha::ViewCtx;

/// Theme colors for navbar
struct NavbarColors {
    bg: Color32,
    text_active: Color32,
    text_inactive: Color32,
    button_active_bg: Color32,
}

impl NavbarColors {
    fn from_dark_mode(is_dark: bool) -> Self {
        if is_dark {
            Self {
                bg: Color32::from_rgb(17, 24, 39),               // bg_primary dark
                text_active: Color32::from_rgb(249, 250, 251),   // text_primary dark
                text_inactive: Color32::from_rgb(156, 163, 175), // text_muted
                button_active_bg: Color32::from_rgb(55, 65, 81), // bg_tertiary dark
            }
        } else {
            Self {
                bg: Color32::from_rgb(31, 41, 55), // dark bar on light theme
                text_active: Color32::WHITE,
                text_inactive: Color32::from_rgb(156, 163, 175),
                button_active_bg: Color32::from_rgb(55, 65, 81),
            }
        }
    }
}

/// A navigation bar component
pub struct Navbar<'a> {
    title: Option<&'a str>,
    show_back: bool,
}

impl<'a> Navbar<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            show_back: false,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_back(mut self) -> Self {
        self.show_back = true;
        self
    }

    /// Show navbar with navigation items
    pub fn show<P, Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        router: &Router<P>,
        items: &[(&str, P)],
        _to_msg: impl Fn(RouterMsg<P>) -> Msg + Clone,
    ) where
        P: Clone + PartialEq,
    {
        let is_dark = ctx.ui.ctx().style().visuals.dark_mode;
        let colors = NavbarColors::from_dark_mode(is_dark);

        egui::Frame::new()
            .fill(colors.bg)
            .inner_margin(egui::Margin::symmetric(16, 8))
            .show(ctx.ui, |ui| {
                ui.horizontal(|ui| {
                    // Back button
                    if self.show_back {
                        let enabled = router.can_back();
                        let back_icon = RichText::new(icons::ARROW_LEFT)
                            .family(egui::FontFamily::Name("icons".into()));
                        if ui
                            .add_enabled(enabled, egui::Button::new(back_icon))
                            .clicked()
                        {
                            // Need to emit outside
                        }
                    }

                    // Title
                    if let Some(title) = self.title {
                        ui.label(RichText::new(title).strong().color(colors.text_active));
                        ui.add_space(16.0);
                    }

                    // Nav items
                    for (label, page) in items {
                        let is_active = router.is_at(page);
                        let text = RichText::new(*label).color(if is_active {
                            colors.text_active
                        } else {
                            colors.text_inactive
                        });

                        let button = if is_active {
                            egui::Button::new(text).fill(colors.button_active_bg)
                        } else {
                            egui::Button::new(text).fill(Color32::TRANSPARENT)
                        };

                        if ui.add(button).clicked() && !is_active {
                            // Store for later emit
                        }
                    }
                });
            });
    }
}

impl<'a> Default for Navbar<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Get primary color for active nav items based on theme
fn nav_active_color(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_rgb(96, 165, 250) // primary dark
    } else {
        Color32::from_rgb(59, 130, 246) // primary light
    }
}

/// Get text color for active nav items based on theme
fn nav_active_text_color(is_dark: bool) -> Color32 {
    if is_dark {
        Color32::from_rgb(17, 24, 39) // dark text on light button
    } else {
        Color32::WHITE
    }
}

/// Simple horizontal nav bar (functional style)
pub fn navbar<P, Msg>(
    ctx: &mut ViewCtx<'_, Msg>,
    router: &Router<P>,
    items: &[(&str, P)],
    to_msg: impl Fn(RouterMsg<P>) -> Msg,
) where
    P: Clone + PartialEq,
{
    let is_dark = ctx.ui.ctx().style().visuals.dark_mode;
    let active_bg = nav_active_color(is_dark);
    let active_text = nav_active_text_color(is_dark);

    let mut nav_action: Option<RouterMsg<P>> = None;

    ctx.ui.horizontal(|ui| {
        // Back button
        if router.can_back() {
            let back_icon =
                RichText::new(icons::ARROW_LEFT).family(egui::FontFamily::Name("icons".into()));
            if ui.button(back_icon).clicked() {
                nav_action = Some(RouterMsg::Back);
            }
        }

        ui.separator();

        // Nav items
        for (label, page) in items {
            let is_active = router.is_at(page);

            let response = if is_active {
                ui.add(egui::Button::new(RichText::new(*label).color(active_text)).fill(active_bg))
            } else {
                ui.button(*label)
            };

            if response.clicked() && !is_active {
                nav_action = Some(RouterMsg::Navigate(page.clone()));
            }
        }
    });

    if let Some(action) = nav_action {
        ctx.emit(to_msg(action));
    }
}

/// Sidebar navigation (vertical)
pub fn sidebar<P, Msg>(
    ctx: &mut ViewCtx<'_, Msg>,
    router: &Router<P>,
    items: &[(&str, P)],
    to_msg: impl Fn(RouterMsg<P>) -> Msg,
) where
    P: Clone + PartialEq,
{
    let is_dark = ctx.ui.ctx().style().visuals.dark_mode;
    let active_bg = nav_active_color(is_dark);
    let active_text = nav_active_text_color(is_dark);

    let mut nav_action: Option<RouterMsg<P>> = None;

    ctx.ui.vertical(|ui| {
        for (label, page) in items {
            let is_active = router.is_at(page);

            let response = if is_active {
                ui.add(
                    egui::Button::new(RichText::new(*label).strong().color(active_text))
                        .fill(active_bg)
                        .min_size(egui::vec2(ui.available_width(), 0.0)),
                )
            } else {
                ui.add(
                    egui::Button::new(RichText::new(*label))
                        .fill(Color32::TRANSPARENT)
                        .min_size(egui::vec2(ui.available_width(), 0.0)),
                )
            };

            if response.clicked() && !is_active {
                nav_action = Some(RouterMsg::Navigate(page.clone()));
            }
        }
    });

    if let Some(action) = nav_action {
        ctx.emit(to_msg(action));
    }
}
