//! Timeline - Seek bar with markers and playhead
//!
//! A timeline component for video/audio with seek, markers, and regions.
//!
//! # Example
//! ```ignore
//! Timeline::new(model.duration)
//!     .position(model.position)
//!     .markers(&model.markers)
//!     .show_with(ctx, |event| match event {
//!         TimelineEvent::Seek(pos) => Msg::Seek(pos),
//!         TimelineEvent::MarkerClick(idx) => Msg::JumpToMarker(idx),
//!     });
//! ```

use crate::Theme;
use egui::{Color32, Rect, Sense, Stroke, Ui, Vec2};
use egui_cha::ViewCtx;

/// Timeline events
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineEvent {
    /// Seek to position (0.0 - 1.0 normalized)
    Seek(f64),
    /// Seek to absolute time in seconds
    SeekAbsolute(f64),
    /// Marker clicked
    MarkerClick(usize),
    /// Region selected (start, end in normalized 0.0-1.0)
    RegionSelect(f64, f64),
}

/// A marker on the timeline
#[derive(Debug, Clone)]
pub struct TimelineMarker {
    /// Position (0.0 - 1.0 normalized, or absolute time if using duration)
    pub position: f64,
    /// Label
    pub label: String,
    /// Color
    pub color: Option<Color32>,
}

impl TimelineMarker {
    /// Create a marker at normalized position
    pub fn new(position: f64, label: impl Into<String>) -> Self {
        Self {
            position,
            label: label.into(),
            color: None,
        }
    }

    /// Create a marker at absolute time (seconds)
    pub fn at_time(time: f64, duration: f64, label: impl Into<String>) -> Self {
        Self {
            position: time / duration,
            label: label.into(),
            color: None,
        }
    }

    /// Set marker color
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

/// A region on the timeline (loop region, selection, etc.)
#[derive(Debug, Clone)]
pub struct TimelineRegion {
    /// Start position (0.0 - 1.0 normalized)
    pub start: f64,
    /// End position (0.0 - 1.0 normalized)
    pub end: f64,
    /// Color
    pub color: Color32,
}

impl TimelineRegion {
    /// Create a region
    pub fn new(start: f64, end: f64, color: Color32) -> Self {
        Self { start, end, color }
    }
}

/// Time display format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeFormat {
    /// Seconds only (e.g., "42.5")
    Seconds,
    /// Minutes:Seconds (e.g., "1:30")
    #[default]
    MinutesSeconds,
    /// Hours:Minutes:Seconds (e.g., "1:30:00")
    HoursMinutesSeconds,
    /// Bars:Beats (for music, requires BPM)
    BarsBeat,
}

/// Timeline component
pub struct Timeline<'a> {
    duration: f64,
    position: f64,
    markers: &'a [TimelineMarker],
    regions: &'a [TimelineRegion],
    height: f32,
    show_time: bool,
    time_format: TimeFormat,
    bpm: Option<f32>,
    show_ticks: bool,
    tick_interval: Option<f64>,
    loop_region: Option<(f64, f64)>,
}

impl<'a> Timeline<'a> {
    /// Create a new timeline with duration in seconds
    pub fn new(duration: f64) -> Self {
        Self {
            duration: duration.max(0.001),
            position: 0.0,
            markers: &[],
            regions: &[],
            height: 32.0,
            show_time: true,
            time_format: TimeFormat::default(),
            bpm: None,
            show_ticks: true,
            tick_interval: None,
            loop_region: None,
        }
    }

    /// Set current position (0.0 - 1.0 normalized)
    pub fn position(mut self, pos: f64) -> Self {
        self.position = pos.clamp(0.0, 1.0);
        self
    }

    /// Set current position in seconds
    pub fn position_seconds(mut self, seconds: f64) -> Self {
        self.position = (seconds / self.duration).clamp(0.0, 1.0);
        self
    }

    /// Set markers
    pub fn markers(mut self, markers: &'a [TimelineMarker]) -> Self {
        self.markers = markers;
        self
    }

    /// Set regions
    pub fn regions(mut self, regions: &'a [TimelineRegion]) -> Self {
        self.regions = regions;
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Show/hide time display
    pub fn show_time(mut self, show: bool) -> Self {
        self.show_time = show;
        self
    }

    /// Set time format
    pub fn time_format(mut self, format: TimeFormat) -> Self {
        self.time_format = format;
        self
    }

    /// Set BPM for musical time display
    pub fn bpm(mut self, bpm: f32) -> Self {
        self.bpm = Some(bpm);
        self
    }

    /// Show/hide tick marks
    pub fn show_ticks(mut self, show: bool) -> Self {
        self.show_ticks = show;
        self
    }

    /// Set tick interval in seconds (auto-calculated if None)
    pub fn tick_interval(mut self, interval: f64) -> Self {
        self.tick_interval = Some(interval);
        self
    }

    /// Set loop region (start, end in normalized 0.0-1.0)
    pub fn loop_region(mut self, start: f64, end: f64) -> Self {
        self.loop_region = Some((start.clamp(0.0, 1.0), end.clamp(0.0, 1.0)));
        self
    }

    /// TEA-style: Show timeline and emit events
    pub fn show_with<Msg>(
        self,
        ctx: &mut ViewCtx<'_, Msg>,
        on_event: impl Fn(TimelineEvent) -> Msg,
    ) {
        if let Some(event) = self.render(ctx.ui) {
            ctx.emit(on_event(event));
        }
    }

    /// Show timeline, returns event if any
    pub fn show(self, ui: &mut Ui) -> Option<TimelineEvent> {
        self.render(ui)
    }

    fn render(self, ui: &mut Ui) -> Option<TimelineEvent> {
        let theme = Theme::current(ui.ctx());
        let mut event = None;

        // Calculate dimensions
        let time_width = if self.show_time { 60.0 } else { 0.0 };
        let available_width = ui.available_width();
        let track_width = available_width - time_width - theme.spacing_sm;

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, self.height),
            Sense::click_and_drag(),
        );

        if !ui.is_rect_visible(rect) {
            return None;
        }

        let track_rect = Rect::from_min_size(
            rect.min + Vec2::new(time_width + theme.spacing_sm, 0.0),
            Vec2::new(track_width, self.height),
        );

        // Handle seek interaction
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                if track_rect.contains(pos) {
                    let normalized = ((pos.x - track_rect.min.x) / track_rect.width()) as f64;
                    let normalized = normalized.clamp(0.0, 1.0);
                    event = Some(TimelineEvent::Seek(normalized));
                }
            }
        }

        // Check marker clicks
        for (idx, marker) in self.markers.iter().enumerate() {
            let marker_x = track_rect.min.x + (marker.position as f32) * track_rect.width();
            let marker_rect = Rect::from_center_size(
                egui::pos2(marker_x, track_rect.center().y),
                Vec2::new(12.0, self.height),
            );

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if marker_rect.contains(pos) {
                        event = Some(TimelineEvent::MarkerClick(idx));
                    }
                }
            }
        }

        let painter = ui.painter();

        // Draw time display
        if self.show_time {
            let time_rect = Rect::from_min_size(rect.min, Vec2::new(time_width, self.height));
            let current_time = self.position * self.duration;
            let time_str = self.format_time(current_time);

            painter.text(
                time_rect.center(),
                egui::Align2::CENTER_CENTER,
                time_str,
                egui::FontId::monospace(theme.font_size_sm),
                theme.text_primary,
            );
        }

        // Draw track background
        painter.rect_filled(track_rect, theme.radius_sm, theme.bg_secondary);

        // Draw regions
        for region in self.regions {
            let start_x = track_rect.min.x + (region.start as f32) * track_rect.width();
            let end_x = track_rect.min.x + (region.end as f32) * track_rect.width();
            let region_rect = Rect::from_min_max(
                egui::pos2(start_x, track_rect.min.y),
                egui::pos2(end_x, track_rect.max.y),
            );
            painter.rect_filled(region_rect, theme.radius_sm * 0.5, region.color);
        }

        // Draw loop region
        if let Some((start, end)) = self.loop_region {
            let start_x = track_rect.min.x + (start as f32) * track_rect.width();
            let end_x = track_rect.min.x + (end as f32) * track_rect.width();
            let loop_rect = Rect::from_min_max(
                egui::pos2(start_x, track_rect.min.y),
                egui::pos2(end_x, track_rect.max.y),
            );
            let loop_color = Color32::from_rgba_unmultiplied(
                theme.primary.r(),
                theme.primary.g(),
                theme.primary.b(),
                40,
            );
            painter.rect_filled(loop_rect, theme.radius_sm * 0.5, loop_color);

            // Loop boundaries
            painter.line_segment(
                [egui::pos2(start_x, track_rect.min.y), egui::pos2(start_x, track_rect.max.y)],
                Stroke::new(2.0, theme.primary),
            );
            painter.line_segment(
                [egui::pos2(end_x, track_rect.min.y), egui::pos2(end_x, track_rect.max.y)],
                Stroke::new(2.0, theme.primary),
            );
        }

        // Draw tick marks
        if self.show_ticks {
            let interval = self.tick_interval.unwrap_or_else(|| self.auto_tick_interval());
            let num_ticks = (self.duration / interval).ceil() as usize;

            for i in 0..=num_ticks {
                let time = i as f64 * interval;
                if time > self.duration {
                    break;
                }
                let x = track_rect.min.x + (time / self.duration) as f32 * track_rect.width();
                let is_major = i % 4 == 0;
                let tick_height = if is_major { 8.0 } else { 4.0 };
                let tick_color = if is_major {
                    theme.text_muted
                } else {
                    Color32::from_rgba_unmultiplied(
                        theme.text_muted.r(),
                        theme.text_muted.g(),
                        theme.text_muted.b(),
                        100,
                    )
                };

                painter.line_segment(
                    [
                        egui::pos2(x, track_rect.max.y - tick_height),
                        egui::pos2(x, track_rect.max.y),
                    ],
                    Stroke::new(1.0, tick_color),
                );
            }
        }

        // Draw markers
        for marker in self.markers {
            let marker_x = track_rect.min.x + (marker.position as f32) * track_rect.width();
            let marker_color = marker.color.unwrap_or(theme.state_warning);

            // Marker line
            painter.line_segment(
                [
                    egui::pos2(marker_x, track_rect.min.y),
                    egui::pos2(marker_x, track_rect.max.y),
                ],
                Stroke::new(2.0, marker_color),
            );

            // Marker triangle at top
            let tri_size = 6.0;
            let points = vec![
                egui::pos2(marker_x - tri_size, track_rect.min.y),
                egui::pos2(marker_x + tri_size, track_rect.min.y),
                egui::pos2(marker_x, track_rect.min.y + tri_size),
            ];
            painter.add(egui::Shape::convex_polygon(
                points,
                marker_color,
                Stroke::NONE,
            ));
        }

        // Draw playhead
        let playhead_x = track_rect.min.x + (self.position as f32) * track_rect.width();

        // Playhead line
        painter.line_segment(
            [
                egui::pos2(playhead_x, track_rect.min.y),
                egui::pos2(playhead_x, track_rect.max.y),
            ],
            Stroke::new(2.0, theme.state_success),
        );

        // Playhead triangle at top
        let head_size = 8.0;
        let head_points = vec![
            egui::pos2(playhead_x - head_size, track_rect.min.y),
            egui::pos2(playhead_x + head_size, track_rect.min.y),
            egui::pos2(playhead_x, track_rect.min.y + head_size),
        ];
        painter.add(egui::Shape::convex_polygon(
            head_points,
            theme.state_success,
            Stroke::NONE,
        ));

        // Draw border
        painter.rect_stroke(
            track_rect,
            theme.radius_sm,
            Stroke::new(theme.border_width, theme.border),
            egui::StrokeKind::Inside,
        );

        event
    }

    fn format_time(&self, seconds: f64) -> String {
        match self.time_format {
            TimeFormat::Seconds => format!("{:.1}", seconds),
            TimeFormat::MinutesSeconds => {
                let mins = (seconds / 60.0).floor() as u32;
                let secs = seconds % 60.0;
                format!("{}:{:05.2}", mins, secs)
            }
            TimeFormat::HoursMinutesSeconds => {
                let hours = (seconds / 3600.0).floor() as u32;
                let mins = ((seconds % 3600.0) / 60.0).floor() as u32;
                let secs = seconds % 60.0;
                format!("{}:{:02}:{:05.2}", hours, mins, secs)
            }
            TimeFormat::BarsBeat => {
                if let Some(bpm) = self.bpm {
                    let beats_per_second = bpm as f64 / 60.0;
                    let total_beats = seconds * beats_per_second;
                    let bars = (total_beats / 4.0).floor() as u32 + 1;
                    let beat = (total_beats % 4.0).floor() as u32 + 1;
                    format!("{}:{}", bars, beat)
                } else {
                    format!("{:.1}", seconds)
                }
            }
        }
    }

    fn auto_tick_interval(&self) -> f64 {
        // Auto-calculate a nice tick interval based on duration
        let target_ticks = 16.0;
        let raw_interval = self.duration / target_ticks;

        // Round to nice values
        let nice_intervals = [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0];
        nice_intervals
            .iter()
            .copied()
            .find(|&i| i >= raw_interval)
            .unwrap_or(60.0)
    }
}
