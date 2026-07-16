//! Navbar molecule - Navigation bar with router integration

use crate::atoms::icons;
use crate::theme::{Theme, ThemeVariant};
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
    fn from_theme(theme: &Theme) -> Self {
        match theme.variant {
            ThemeVariant::Dark => Self {
                bg: theme.bg_primary,
                text_active: theme.text_primary,
                text_inactive: theme.text_muted,
                button_active_bg: theme.bg_tertiary,
            },
            // The bar keeps a deliberately dark look on the light theme, so
            // light-theme text tokens (dark on light) would be unreadable
            // here; the bar colors stay fixed.
            ThemeVariant::Light => Self {
                bg: Color32::from_rgb(31, 41, 55),
                text_active: Color32::WHITE,
                text_inactive: theme.text_muted,
                button_active_bg: Color32::from_rgb(55, 65, 81),
            },
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
        let theme = Theme::current(ctx.ui.ctx());
        let colors = NavbarColors::from_theme(&theme);

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

/// Simple horizontal nav bar (functional style)
pub fn navbar<P, Msg>(
    ctx: &mut ViewCtx<'_, Msg>,
    router: &Router<P>,
    items: &[(&str, P)],
    to_msg: impl Fn(RouterMsg<P>) -> Msg,
) where
    P: Clone + PartialEq,
{
    let theme = Theme::current(ctx.ui.ctx());
    let active_bg = theme.primary;
    let active_text = theme.primary_text;

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
    let theme = Theme::current(ctx.ui.ctx());
    let active_bg = theme.primary;
    let active_text = theme.primary_text;

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
