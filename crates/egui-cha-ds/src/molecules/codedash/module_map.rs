//! Force-directed module map for codedash analyze results.
//!
//! Renders a zoomable, pannable graph of module bubbles with dependency edges.
//! Each node is a [`MetricsBubble`] laid out via a simple force simulation.
//!
//! This is the egui equivalent of `codedash view`'s HTML SVG graph.
//!
//! # Example
//!
//! ```ignore
//! let mut state = ModuleMapState::new(&result);
//! ModuleMap::new(&result)
//!     .state(&mut state)
//!     .show(ui);
//! ```

use std::collections::HashMap;

use codedash_schemas::analyze::AnalyzeResult;
use egui::{Color32, Pos2, Ui, Vec2};

use super::{size_to_radius, MetricsBubble};

/// Domain palette matching codedash's dark-theme colors.
const PALETTE: &[Color32] = &[
    Color32::from_rgb(88, 166, 255),  // blue
    Color32::from_rgb(126, 231, 135), // green
    Color32::from_rgb(240, 136, 62),  // orange
    Color32::from_rgb(210, 168, 255), // purple
    Color32::from_rgb(121, 192, 255), // light blue
    Color32::from_rgb(247, 120, 186), // pink
    Color32::from_rgb(255, 166, 87),  // light orange
    Color32::from_rgb(86, 212, 221),  // cyan
    Color32::from_rgb(212, 151, 108), // tan
    Color32::from_rgb(165, 214, 255), // pale blue
];

/// Internal node for the force simulation.
#[derive(Clone)]
struct SimNode {
    /// Index into AnalyzeResult.entries
    entry_idx: usize,
    /// Module file path (grouping key)
    file: String,
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    domain_color: Color32,
    fixed: bool,
}

/// Internal edge for rendering.
struct SimEdge {
    source_idx: usize,
    target_idx: usize,
}

/// Persistent state for the module map simulation.
pub struct ModuleMapState {
    nodes: Vec<SimNode>,
    edges: Vec<SimEdge>,
    alpha: f32,
    zoom: f32,
    pan: Vec2,
    hovered_node: Option<usize>,
    initialized: bool,
}

impl Default for ModuleMapState {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            alpha: 1.0,
            zoom: 1.0,
            pan: Vec2::ZERO,
            hovered_node: None,
            initialized: false,
        }
    }
}

impl ModuleMapState {
    /// Initialize state from an AnalyzeResult. Call once, or when data changes.
    pub fn init(&mut self, result: &AnalyzeResult) {
        // Group entries by file → assign domain colors
        let mut file_domains: HashMap<String, usize> = HashMap::new();
        let mut domain_counter = 0usize;

        for (idx, entry) in result.entries.iter().enumerate() {
            let domain_idx = *file_domains.entry(entry.file.clone()).or_insert_with(|| {
                let d = domain_counter;
                domain_counter += 1;
                d
            });

            let radius = size_to_radius(entry.normalized.size, 16.0, 48.0);
            let color = PALETTE[domain_idx % PALETTE.len()];

            // Initial position: spread by domain in a circle
            let angle = (domain_idx as f32 / domain_counter.max(1) as f32) * std::f32::consts::TAU
                - std::f32::consts::FRAC_PI_2;
            let spread = 200.0;
            let jitter_x = ((idx * 7 + 13) % 100) as f32 / 100.0 - 0.5;
            let jitter_y = ((idx * 11 + 29) % 100) as f32 / 100.0 - 0.5;

            self.nodes.push(SimNode {
                entry_idx: idx,
                file: entry.file.clone(),
                pos: Vec2::new(
                    angle.cos() * spread + jitter_x * spread * 0.4,
                    angle.sin() * spread + jitter_y * spread * 0.4,
                ),
                vel: Vec2::ZERO,
                radius,
                domain_color: color,
                fixed: false,
            });
        }

        // Build edges: entries in the same file are implicitly connected.
        // For a first version, connect entries that share the same file.
        let mut file_node_indices: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            file_node_indices
                .entry(node.file.clone())
                .or_default()
                .push(i);
        }

        // Connect first node of each file to each other (module-level edges)
        let file_heads: Vec<usize> = file_node_indices
            .values()
            .filter_map(|indices| indices.first().copied())
            .collect();

        // Simple heuristic: connect files that share similar domain grouping
        for i in 0..file_heads.len() {
            for j in (i + 1)..file_heads.len() {
                let a = &self.nodes[file_heads[i]];
                let b = &self.nodes[file_heads[j]];
                // Connect if same domain (same file prefix up to first /)
                let a_prefix = a.file.split('/').next().unwrap_or("");
                let b_prefix = b.file.split('/').next().unwrap_or("");
                if a_prefix == b_prefix && !a_prefix.is_empty() {
                    self.edges.push(SimEdge {
                        source_idx: file_heads[i],
                        target_idx: file_heads[j],
                    });
                }
            }
        }

        self.alpha = 1.0;
        self.initialized = true;
    }

    /// Run one tick of the force simulation.
    fn tick(&mut self) {
        if self.alpha < 0.005 {
            return;
        }

        let n = self.nodes.len();
        let alpha = self.alpha;

        // Charge repulsion
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.nodes[j].pos.x - self.nodes[i].pos.x;
                let dy = self.nodes[j].pos.y - self.nodes[i].pos.y;
                let d2 = (dx * dx + dy * dy).max(1.0);
                let d = d2.sqrt();

                let same_file = self.nodes[i].file == self.nodes[j].file;
                let strength = if same_file { -300.0 } else { -600.0 };
                let f = strength / d2 * alpha;
                let fx = dx / d * f;
                let fy = dy / d * f;

                if !self.nodes[i].fixed {
                    self.nodes[i].vel.x -= fx;
                    self.nodes[i].vel.y -= fy;
                }
                if !self.nodes[j].fixed {
                    self.nodes[j].vel.x += fx;
                    self.nodes[j].vel.y += fy;
                }
            }
        }

        // Link attraction
        for edge in &self.edges {
            let (si, ti) = (edge.source_idx, edge.target_idx);
            let dx = self.nodes[ti].pos.x - self.nodes[si].pos.x;
            let dy = self.nodes[ti].pos.y - self.nodes[si].pos.y;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            let f = (d - 120.0) * 0.003 * alpha;
            let fx = dx / d * f;
            let fy = dy / d * f;

            if !self.nodes[si].fixed {
                self.nodes[si].vel.x += fx;
                self.nodes[si].vel.y += fy;
            }
            if !self.nodes[ti].fixed {
                self.nodes[ti].vel.x -= fx;
                self.nodes[ti].vel.y -= fy;
            }
        }

        // Center gravity
        for node in &mut self.nodes {
            if node.fixed {
                continue;
            }
            node.vel.x -= node.pos.x * 0.001 * alpha;
            node.vel.y -= node.pos.y * 0.001 * alpha;
        }

        // Collision
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.nodes[j].pos.x - self.nodes[i].pos.x;
                let dy = self.nodes[j].pos.y - self.nodes[i].pos.y;
                let d = (dx * dx + dy * dy).sqrt().max(0.1);
                let min_d = self.nodes[i].radius + self.nodes[j].radius + 8.0;
                if d < min_d {
                    let push = (min_d - d) / d * 0.5;
                    if !self.nodes[i].fixed {
                        self.nodes[i].vel.x -= dx * push;
                        self.nodes[i].vel.y -= dy * push;
                    }
                    if !self.nodes[j].fixed {
                        self.nodes[j].vel.x += dx * push;
                        self.nodes[j].vel.y += dy * push;
                    }
                }
            }
        }

        // Apply velocities
        let damping = 0.55;
        for node in &mut self.nodes {
            if node.fixed {
                continue;
            }
            node.vel *= damping;
            node.pos += node.vel;
        }

        self.alpha *= 1.0 - 0.012;
        self.alpha = self.alpha.max(0.001);
    }
}

/// Force-directed module map widget.
pub struct ModuleMap<'a> {
    result: &'a AnalyzeResult,
}

impl<'a> ModuleMap<'a> {
    /// Create a new module map for the given analyze result.
    pub fn new(result: &'a AnalyzeResult) -> Self {
        Self { result }
    }

    /// Show the module map with the given persistent state.
    pub fn show(&self, ui: &mut Ui, state: &mut ModuleMapState) {
        if !state.initialized {
            state.init(self.result);
        }

        // Run simulation while cooling
        if state.alpha > 0.005 {
            state.tick();
            ui.ctx().request_repaint();
        }

        let max_churn = self
            .result
            .entries
            .iter()
            .filter_map(|e| e.git_churn_30d)
            .max()
            .unwrap_or(1);

        let available = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available, egui::Sense::click_and_drag());

        // Zoom
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 && response.hovered() {
            let factor = if scroll_delta > 0.0 { 1.08 } else { 0.92 };
            state.zoom = (state.zoom * factor).clamp(0.15, 4.0);
        }

        // Pan
        if response.dragged() {
            state.pan += response.drag_delta();
        }

        let painter = ui.painter_at(available);
        let center = available.center().to_vec2() + state.pan;
        let zoom = state.zoom;

        // Draw edges
        let edge_color = Color32::from_rgba_unmultiplied(48, 54, 61, 115);
        for edge in &state.edges {
            let a = &state.nodes[edge.source_idx];
            let b = &state.nodes[edge.target_idx];
            let p1 = Pos2::new(center.x + a.pos.x * zoom, center.y + a.pos.y * zoom);
            let p2 = Pos2::new(center.x + b.pos.x * zoom, center.y + b.pos.y * zoom);
            painter.line_segment([p1, p2], egui::Stroke::new(1.2, edge_color));
        }

        // Draw nodes
        state.hovered_node = None;
        for (i, node) in state.nodes.iter().enumerate() {
            let screen_pos = Pos2::new(center.x + node.pos.x * zoom, center.y + node.pos.y * zoom);

            let entry = &self.result.entries[node.entry_idx];
            let bubble = MetricsBubble::new(entry)
                .domain_color(node.domain_color)
                .max_churn(max_churn)
                .radius_range(16.0 * zoom, 48.0 * zoom)
                .show_label(zoom > 0.4);

            let bounds = bubble.paint_at(&painter, screen_pos);

            // Hit test
            if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                if bounds.contains(pointer) {
                    state.hovered_node = Some(i);
                }
            }
        }

        // Tooltip for hovered node
        if let Some(idx) = state.hovered_node {
            let entry = &self.result.entries[state.nodes[idx].entry_idx];
            response.clone().on_hover_ui(|ui| {
                let bubble = MetricsBubble::new(entry);
                bubble.paint_tooltip(ui);
            });
        }
    }
}
