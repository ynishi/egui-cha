//! Oscilloscope atom - Real-time signal visualization
//!
//! Displays real-time audio/signal data in oscilloscope style with
//! trigger, grid, and various display modes.
//!
//! # Example
//! ```ignore
//! // Basic oscilloscope
//! Oscilloscope::new(&signal_buffer)
//!     .show(ui);
//!
//! // With grid and trigger
//! Oscilloscope::new(&signal_buffer)
//!     .grid(true)
//!     .trigger_level(0.0)
//!     .show(ui);
//! ```

use crate::Theme;
use egui::{Color32, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Oscilloscope display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeMode {
    /// Standard line display
    #[default]
    Line,
    /// Filled area below the line
    Filled,
    /// Dot display (sample points)
    Dots,
    /// XY mode (Lissajous)
    XY,
}

/// Oscilloscope trigger mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TriggerMode {
    /// No triggering (free running)
    #[default]
    Free,
    /// Trigger on rising edge
    Rising,
    /// Trigger on falling edge
    Falling,
}

/// Real-time oscilloscope display
pub struct Oscilloscope<'a> {
    /// Primary signal buffer (normalized -1.0 to 1.0)
    samples: &'a [f32],
    /// Secondary signal for XY mode
    samples_y: Option<&'a [f32]>,
    width: Option<f32>,
    height: f32,
    mode: ScopeMode,
    show_grid: bool,
    grid_divisions: usize,
    trigger_mode: TriggerMode,
    trigger_level: f32,
    color: Option<Color32>,
    phosphor_glow: bool,
    line_width: f32,
}

impl<'a> Oscilloscope<'a> {
    /// Create a new oscilloscope with the given sample buffer
    ///
    /// Samples should be normalized to -1.0..1.0 range
    pub fn new(samples: &'a [f32]) -> Self {
        Self {
            samples,
            samples_y: None,
            width: None,
            height: 120.0,
            mode: ScopeMode::default(),
            show_grid: true,
            grid_divisions: 8,
            trigger_mode: TriggerMode::default(),
            trigger_level: 0.0,
            color: None,
            phosphor_glow: false,
            line_width: 1.5,
        }
    }

    /// Set custom width (defaults to available width)
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set the display mode
    pub fn mode(mut self, mode: ScopeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Use filled mode
    pub fn filled(mut self) -> Self {
        self.mode = ScopeMode::Filled;
        self
    }

    /// Use dots mode
    pub fn dots(mut self) -> Self {
        self.mode = ScopeMode::Dots;
        self
    }

    /// Set XY mode with secondary signal
    pub fn xy(mut self, samples_y: &'a [f32]) -> Self {
        self.mode = ScopeMode::XY;
        self.samples_y = Some(samples_y);
        self
    }

    /// Show/hide grid
    pub fn grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Set number of grid divisions
    pub fn grid_divisions(mut self, divisions: usize) -> Self {
        self.grid_divisions = divisions;
        self
    }

    /// Set trigger mode
    pub fn trigger(mut self, mode: TriggerMode) -> Self {
        self.trigger_mode = mode;
        self
    }

    /// Set trigger level (-1.0 to 1.0)
    pub fn trigger_level(mut self, level: f32) -> Self {
        self.trigger_level = level.clamp(-1.0, 1.0);
        self
    }

    /// Set custom trace color
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Enable phosphor glow effect (retro CRT look)
    pub fn phosphor(mut self, enabled: bool) -> Self {
        self.phosphor_glow = enabled;
        self
    }

    /// Set line width
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Find trigger point in samples
    fn find_trigger_point(&self) -> usize {
        if self.samples.len() < 2 {
            return 0;
        }

        match self.trigger_mode {
            TriggerMode::Free => 0,
            TriggerMode::Rising => {
                // Find rising edge crossing trigger level
                for i in 1..self.samples.len() {
                    if self.samples[i - 1] < self.trigger_level
                        && self.samples[i] >= self.trigger_level
                    {
                        return i;
                    }
                }
                0
            }
            TriggerMode::Falling => {
                // Find falling edge crossing trigger level
                for i in 1..self.samples.len() {
                    if self.samples[i - 1] > self.trigger_level
                        && self.samples[i] <= self.trigger_level
                    {
                        return i;
                    }
                }
                0
            }
        }
    }

    /// Display the oscilloscope
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());

        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(width, self.height),
            Sense::hover(),
        );

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter();

        // Background
        let bg_color = if self.phosphor_glow {
            Color32::from_rgb(10, 20, 15) // Dark green-ish for CRT effect
        } else {
            theme.bg_tertiary
        };
        painter.rect_filled(rect, theme.radius_sm, bg_color);

        // Grid
        if self.show_grid {
            self.draw_grid(painter, rect, &theme);
        }

        // Border
        painter.rect_stroke(
            rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Outside,
        );

        // Trace color
        let trace_color = self.color.unwrap_or_else(|| {
            if self.phosphor_glow {
                Color32::from_rgb(100, 255, 150) // Phosphor green
            } else {
                theme.primary
            }
        });

        // Draw based on mode
        match self.mode {
            ScopeMode::XY => self.draw_xy(painter, rect, trace_color),
            _ => self.draw_trace(painter, rect, trace_color),
        }

        response
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect, theme: &Theme) {
        let grid_color = if self.phosphor_glow {
            Color32::from_rgba_unmultiplied(100, 255, 150, 30)
        } else {
            Color32::from_rgba_unmultiplied(
                theme.border.r(),
                theme.border.g(),
                theme.border.b(),
                60,
            )
        };

        let stroke = Stroke::new(0.5, grid_color);

        // Vertical lines
        for i in 1..self.grid_divisions {
            let x = rect.min.x + (rect.width() * i as f32 / self.grid_divisions as f32);
            painter.vline(x, rect.min.y..=rect.max.y, stroke);
        }

        // Horizontal lines
        let v_divisions = (self.grid_divisions / 2).max(2);
        for i in 1..v_divisions {
            let y = rect.min.y + (rect.height() * i as f32 / v_divisions as f32);
            painter.hline(rect.min.x..=rect.max.x, y, stroke);
        }

        // Center line (brighter)
        let center_color = if self.phosphor_glow {
            Color32::from_rgba_unmultiplied(100, 255, 150, 50)
        } else {
            Color32::from_rgba_unmultiplied(
                theme.border.r(),
                theme.border.g(),
                theme.border.b(),
                100,
            )
        };
        let center_y = rect.center().y;
        painter.hline(rect.min.x..=rect.max.x, center_y, Stroke::new(0.5, center_color));
    }

    fn draw_trace(&self, painter: &egui::Painter, rect: Rect, color: Color32) {
        if self.samples.is_empty() {
            return;
        }

        let trigger_offset = self.find_trigger_point();

        // Calculate how many samples to display
        let display_samples = self.samples.len().saturating_sub(trigger_offset);
        if display_samples == 0 {
            return;
        }

        let samples_to_use = &self.samples[trigger_offset..];
        let step = (samples_to_use.len() as f32 / rect.width()).max(1.0);

        match self.mode {
            ScopeMode::Line => {
                // Draw as connected line
                let points: Vec<egui::Pos2> = (0..rect.width() as usize)
                    .map(|x| {
                        let sample_idx = ((x as f32 * step) as usize).min(samples_to_use.len() - 1);
                        let sample = samples_to_use[sample_idx];
                        let y = rect.center().y - (sample * rect.height() / 2.0);
                        egui::pos2(rect.min.x + x as f32, y)
                    })
                    .collect();

                if self.phosphor_glow {
                    // Draw glow layer
                    let glow_color = Color32::from_rgba_unmultiplied(
                        color.r(), color.g(), color.b(), 40,
                    );
                    painter.add(egui::Shape::line(
                        points.clone(),
                        Stroke::new(self.line_width * 3.0, glow_color),
                    ));
                }

                painter.add(egui::Shape::line(
                    points,
                    Stroke::new(self.line_width, color),
                ));
            }
            ScopeMode::Filled => {
                // Draw filled area
                let mut points: Vec<egui::Pos2> = vec![egui::pos2(rect.min.x, rect.center().y)];

                for x in 0..rect.width() as usize {
                    let sample_idx = ((x as f32 * step) as usize).min(samples_to_use.len() - 1);
                    let sample = samples_to_use[sample_idx];
                    let y = rect.center().y - (sample * rect.height() / 2.0);
                    points.push(egui::pos2(rect.min.x + x as f32, y));
                }

                points.push(egui::pos2(rect.max.x, rect.center().y));

                let fill_color = Color32::from_rgba_unmultiplied(
                    color.r(), color.g(), color.b(), 80,
                );
                painter.add(egui::Shape::convex_polygon(
                    points.clone(),
                    fill_color,
                    Stroke::NONE,
                ));

                // Draw line on top
                let line_points: Vec<egui::Pos2> = points[1..points.len()-1].to_vec();
                painter.add(egui::Shape::line(
                    line_points,
                    Stroke::new(self.line_width, color),
                ));
            }
            ScopeMode::Dots => {
                // Draw as dots
                for x in 0..rect.width() as usize {
                    let sample_idx = ((x as f32 * step) as usize).min(samples_to_use.len() - 1);
                    let sample = samples_to_use[sample_idx];
                    let y = rect.center().y - (sample * rect.height() / 2.0);
                    let pos = egui::pos2(rect.min.x + x as f32, y);

                    if self.phosphor_glow {
                        let glow_color = Color32::from_rgba_unmultiplied(
                            color.r(), color.g(), color.b(), 40,
                        );
                        painter.circle_filled(pos, self.line_width * 2.0, glow_color);
                    }
                    painter.circle_filled(pos, self.line_width, color);
                }
            }
            ScopeMode::XY => unreachable!(),
        }
    }

    fn draw_xy(&self, painter: &egui::Painter, rect: Rect, color: Color32) {
        let Some(samples_y) = self.samples_y else {
            return;
        };

        if self.samples.is_empty() || samples_y.is_empty() {
            return;
        }

        let len = self.samples.len().min(samples_y.len());

        let points: Vec<egui::Pos2> = (0..len)
            .map(|i| {
                let x = rect.center().x + (self.samples[i] * rect.width() / 2.0);
                let y = rect.center().y - (samples_y[i] * rect.height() / 2.0);
                egui::pos2(x, y)
            })
            .collect();

        if self.phosphor_glow {
            // Draw glow
            let glow_color = Color32::from_rgba_unmultiplied(
                color.r(), color.g(), color.b(), 30,
            );
            painter.add(egui::Shape::line(
                points.clone(),
                Stroke::new(self.line_width * 4.0, glow_color),
            ));
        }

        painter.add(egui::Shape::line(
            points,
            Stroke::new(self.line_width, color),
        ));
    }
}
