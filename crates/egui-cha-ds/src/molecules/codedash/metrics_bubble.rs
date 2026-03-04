//! Single-node visual element for codedash metrics.
//!
//! Draws a circle with layered visual encodings matching `codedash view`:
//! - **Body**: filled circle, radius ∝ lines (size percept), color = domain
//! - **Inner ring**: stroke colored by complexity hue (green→red)
//! - **Outer ring**: amber stroke ∝ git churn
//! - **Label**: module/function name
//!
//! # Example
//!
//! ```ignore
//! MetricsBubble::new(&entry)
//!     .domain_color(Color32::from_rgb(88, 166, 255))
//!     .max_churn(30)
//!     .show(ui);
//! ```

use codedash_schemas::analyze::EvalEntry;
use egui::{Color32, Pos2, Ui, Vec2};

use super::{hue_to_color, size_to_radius};

/// A single metrics bubble drawn via custom painting.
pub struct MetricsBubble<'a> {
    entry: &'a EvalEntry,
    domain_color: Color32,
    max_churn: u32,
    min_radius: f32,
    max_radius: f32,
    show_label: bool,
}

impl<'a> MetricsBubble<'a> {
    /// Create a bubble for the given entry.
    pub fn new(entry: &'a EvalEntry) -> Self {
        Self {
            entry,
            domain_color: Color32::from_rgb(139, 148, 158), // default gray
            max_churn: 1,
            min_radius: 16.0,
            max_radius: 48.0,
            show_label: true,
        }
    }

    /// Set the domain color for the body fill.
    pub fn domain_color(mut self, color: Color32) -> Self {
        self.domain_color = color;
        self
    }

    /// Set the max churn value for normalizing the outer ring width.
    pub fn max_churn(mut self, max: u32) -> Self {
        self.max_churn = max.max(1);
        self
    }

    /// Set the radius range (min, max) in pixels.
    pub fn radius_range(mut self, min: f32, max: f32) -> Self {
        self.min_radius = min;
        self.max_radius = max;
        self
    }

    /// Whether to show the label text.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Compute the body radius from the normalized size percept.
    fn body_radius(&self) -> f32 {
        size_to_radius(self.entry.normalized.size, self.min_radius, self.max_radius)
    }

    /// Draw the bubble at a specific position on the painter.
    ///
    /// Returns the bounding rect for hit-testing.
    pub fn paint_at(&self, painter: &egui::Painter, center: Pos2) -> egui::Rect {
        let r = self.body_radius();
        let hue = self.entry.percept.hue;
        let hue_color = hue_to_color(hue);
        let cyclomatic = self.entry.cyclomatic.unwrap_or(0);
        let churn = self.entry.git_churn_30d.unwrap_or(0);

        // Inner ring: complexity
        let ring_w = (cyclomatic as f32 * 0.5).clamp(2.0, 8.0);
        painter.circle_stroke(
            center,
            r + ring_w / 2.0,
            egui::Stroke::new(ring_w, hue_color),
        );

        // Outer ring: churn (amber)
        if churn > 0 {
            let churn_norm = churn as f32 / self.max_churn as f32;
            let cw = (2.0 + churn_norm * 8.0).clamp(2.0, 10.0);
            let alpha = ((0.35 + churn_norm * 0.45) * 255.0) as u8;
            let churn_color = Color32::from_rgba_unmultiplied(240, 136, 62, alpha);
            painter.circle_stroke(
                center,
                r + ring_w + cw / 2.0 + 1.0,
                egui::Stroke::new(cw, churn_color),
            );
        }

        // Body
        painter.circle_filled(center, r, self.domain_color.gamma_multiply(0.85));

        // Label
        if self.show_label {
            let name = short_name(&self.entry.name);
            let font = egui::FontId::proportional(r.clamp(10.0, 14.0) * 0.85);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                name,
                font,
                Color32::WHITE,
            );
        }

        // Bounding rect (including outer ring)
        let total_r = r + ring_w + 12.0;
        egui::Rect::from_center_size(center, Vec2::splat(total_r * 2.0))
    }

    /// Allocate space and draw inline in a Ui.
    ///
    /// Returns the Response for hover/click detection.
    pub fn show(&self, ui: &mut Ui) -> egui::Response {
        let r = self.body_radius();
        let total = r + 20.0; // padding for rings
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(total * 2.0), egui::Sense::click());

        if ui.is_rect_visible(rect) {
            self.paint_at(ui.painter(), rect.center());
        }

        response.clone().on_hover_ui(|ui| {
            self.paint_tooltip(ui);
        });

        response
    }

    /// Render tooltip content for this entry.
    pub fn paint_tooltip(&self, ui: &mut Ui) {
        let e = self.entry;
        ui.strong(&e.full_name);
        ui.separator();

        egui::Grid::new(ui.next_auto_id())
            .num_columns(2)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                row(ui, "Kind", &e.kind);
                row(ui, "Lines", &e.lines.to_string());
                if let Some(cc) = e.cyclomatic {
                    row(ui, "Cyclomatic", &cc.to_string());
                }
                if let Some(p) = e.params {
                    row(ui, "Params", &p.to_string());
                }
                if let Some(d) = e.depth {
                    row(ui, "Depth", &d.to_string());
                }
                if let Some(ch) = e.git_churn_30d {
                    row(ui, "Git churn (30d)", &ch.to_string());
                }
                if let Some(cov) = e.coverage {
                    row(ui, "Coverage", &format!("{:.0}%", cov * 100.0));
                }
                row(ui, "Visibility", &e.visibility);
            });
    }
}

fn row(ui: &mut Ui, label: &str, value: &str) {
    ui.label(label);
    ui.label(value);
    ui.end_row();
}

/// Shorten a name for display inside a bubble (last 2 `::` segments).
fn short_name(name: &str) -> &str {
    // "foo::bar::baz" → "bar::baz", "main" → "main"
    let mut sep_count = 0;
    // Find `::` separators from the right
    let bytes = name.as_bytes();
    let mut i = bytes.len();
    while i >= 2 {
        i -= 1;
        if bytes[i] == b':' && i > 0 && bytes[i - 1] == b':' {
            sep_count += 1;
            if sep_count == 2 {
                return &name[i + 1..];
            }
            // skip the first ':'
            i -= 1;
        }
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_name_simple() {
        assert_eq!(short_name("main"), "main");
        assert_eq!(short_name("foo::bar"), "foo::bar");
        assert_eq!(short_name("a::b::c"), "b::c");
        assert_eq!(short_name("w::x::y::z"), "y::z");
    }
}
