//! codedash metrics visualization components.
//!
//! Renders [`codedash_schemas::analyze::AnalyzeResult`] data as interactive
//! egui widgets. These components are the native GUI equivalent of
//! `codedash view`'s HTML module map.
//!
//! # Components
//!
//! - [`MetricsBubble`] — Single node rendered as a circle with visual encodings
//!   (size, hue ring, churn ring, domain color).
//! - [`MetricsTable`] — Sortable table of evaluated entries with percept-colored cells.
//! - [`ModuleMap`] — Force-directed graph layout of module bubbles with dependency edges.
//!
//! # Feature gate
//!
//! All types require the `codedash` feature:
//!
//! ```toml
//! egui-cha-ds = { version = "0.5", features = ["codedash"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use codedash_schemas::analyze::AnalyzeResult;
//! use egui_cha_ds::codedash::{MetricsTable, ModuleMap};
//!
//! let result: AnalyzeResult = serde_json::from_str(&json)?;
//! // Table view
//! MetricsTable::new(&result).show(ui);
//! // Graph view
//! ModuleMap::new(&result).show(ui);
//! ```

mod metrics_bubble;
mod metrics_table;
mod module_map;

pub use metrics_bubble::MetricsBubble;
pub use metrics_table::{MetricsTable, SortColumn, SortOrder};
pub use module_map::{ModuleMap, ModuleMapState};

use codedash_schemas::analyze::PerceptValues;

/// Map a percept hue value (0–120 green→red) to an egui Color32.
///
/// codedash uses hue 120=green (low complexity) → 0=red (high complexity).
pub fn hue_to_color(hue: f64) -> egui::Color32 {
    let h = (hue.clamp(0.0, 120.0) / 120.0) as f32; // 0.0=red, 1.0=green
    let r = ((1.0 - h) * 255.0) as u8;
    let g = (h * 255.0) as u8;
    egui::Color32::from_rgb(r, g, 80)
}

/// Map a normalized value (0.0–1.0) to a radius in pixels.
pub fn size_to_radius(size_normalized: f64, min_r: f32, max_r: f32) -> f32 {
    let t = size_normalized.clamp(0.0, 1.0) as f32;
    min_r + t * (max_r - min_r)
}

/// Extract the hue value from percept values (complexity encoding).
pub fn percept_hue(percept: &PerceptValues) -> f64 {
    percept.hue
}
