//! Sortable table for codedash analyze results.
//!
//! Displays [`EvalEntry`] items in a table with columns for each metric.
//! Cells are color-coded using percept hue values. Click column headers to sort.
//!
//! # Example
//!
//! ```ignore
//! let mut state = MetricsTableState::default();
//! MetricsTable::new(&result)
//!     .state(&mut state)
//!     .show(ui);
//! ```

use codedash_schemas::analyze::{AnalyzeResult, EvalEntry};
use egui::{Color32, RichText, Ui};

use super::hue_to_color;
use crate::Theme;

/// Which column to sort by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Name,
    Kind,
    Lines,
    Cyclomatic,
    Params,
    Depth,
    Churn,
    Coverage,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

impl SortOrder {
    fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    fn arrow(self) -> &'static str {
        match self {
            Self::Ascending => " ^",
            Self::Descending => " v",
        }
    }
}

/// Sortable metrics table for AnalyzeResult.
pub struct MetricsTable<'a> {
    result: &'a AnalyzeResult,
    sort_col: SortColumn,
    sort_order: SortOrder,
    max_rows: usize,
}

impl<'a> MetricsTable<'a> {
    /// Create a new table for the given analyze result.
    pub fn new(result: &'a AnalyzeResult) -> Self {
        Self {
            result,
            sort_col: SortColumn::Lines,
            sort_order: SortOrder::Descending,
            max_rows: 100,
        }
    }

    /// Set the initial sort column and order.
    pub fn sort(mut self, col: SortColumn, order: SortOrder) -> Self {
        self.sort_col = col;
        self.sort_order = order;
        self
    }

    /// Limit displayed rows.
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n;
        self
    }

    /// Show the table. Returns the updated (sort_col, sort_order) for caller to persist.
    pub fn show(mut self, ui: &mut Ui) -> (SortColumn, SortOrder) {
        let theme = Theme::current(ui.ctx());

        // Summary line
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} entries, {} total",
                    self.result.entries.len(),
                    self.result.total
                ))
                .color(theme.text_muted),
            );
            if self.result.excluded > 0 {
                ui.label(
                    RichText::new(format!("({} excluded)", self.result.excluded))
                        .color(theme.text_muted),
                );
            }
        });

        ui.add_space(4.0);

        // Sort entries
        let mut indices: Vec<usize> = (0..self.result.entries.len()).collect();
        sort_indices(
            &mut indices,
            &self.result.entries,
            self.sort_col,
            self.sort_order,
        );
        indices.truncate(self.max_rows);

        // Table
        egui::Frame::new()
            .stroke(egui::Stroke::new(theme.border_width, theme.border))
            .corner_radius(theme.radius_md)
            .show(ui, |ui| {
                egui::Grid::new(ui.next_auto_id())
                    .num_columns(8)
                    .striped(true)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        // Header
                        self.header_cell(ui, "Name", SortColumn::Name);
                        self.header_cell(ui, "Kind", SortColumn::Kind);
                        self.header_cell(ui, "Lines", SortColumn::Lines);
                        self.header_cell(ui, "CC", SortColumn::Cyclomatic);
                        self.header_cell(ui, "Params", SortColumn::Params);
                        self.header_cell(ui, "Depth", SortColumn::Depth);
                        self.header_cell(ui, "Churn", SortColumn::Churn);
                        self.header_cell(ui, "Cov", SortColumn::Coverage);
                        ui.end_row();

                        // Rows
                        for &idx in &indices {
                            let entry = &self.result.entries[idx];
                            self.entry_row(ui, entry);
                        }
                    });
            });

        (self.sort_col, self.sort_order)
    }

    fn header_cell(&mut self, ui: &mut Ui, label: &str, col: SortColumn) {
        let theme = Theme::current(ui.ctx());
        let text = if self.sort_col == col {
            format!("{}{}", label, self.sort_order.arrow())
        } else {
            label.to_string()
        };

        let resp = egui::Frame::new()
            .fill(theme.bg_secondary)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .corner_radius(theme.radius_sm)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(text)
                        .strong()
                        .size(11.0)
                        .color(theme.text_primary),
                )
            })
            .inner;

        if resp.interact(egui::Sense::click()).clicked() {
            if self.sort_col == col {
                self.sort_order = self.sort_order.toggle();
            } else {
                self.sort_col = col;
                self.sort_order = SortOrder::Descending;
            }
        }
    }

    fn entry_row(&self, ui: &mut Ui, entry: &EvalEntry) {
        // Name (truncated)
        let name = if entry.name.len() > 30 {
            format!("{}...", &entry.name[..27])
        } else {
            entry.name.clone()
        };
        ui.label(&name).on_hover_ui(|ui| {
            ui.label(&entry.full_name);
            ui.label(format!(
                "{}:{}-{}",
                entry.file, entry.start_line, entry.end_line
            ));
        });

        // Kind
        ui.label(RichText::new(&entry.kind).size(11.0));

        // Lines — colored by size percept
        let hue = entry.percept.hue;
        let color = hue_to_color(hue);
        ui.label(RichText::new(entry.lines.to_string()).color(color));

        // Cyclomatic
        metric_cell(ui, entry.cyclomatic, hue);

        // Params
        metric_cell(ui, entry.params, 120.0); // neutral

        // Depth
        metric_cell(ui, entry.depth, hue);

        // Churn
        if let Some(ch) = entry.git_churn_30d {
            let churn_color = if ch > 10 {
                Color32::from_rgb(240, 136, 62) // amber
            } else {
                Color32::GRAY
            };
            ui.label(RichText::new(ch.to_string()).color(churn_color));
        } else {
            ui.label("—");
        }

        // Coverage
        if let Some(cov) = entry.coverage {
            let pct = (cov * 100.0) as u32;
            let cov_color = if cov >= 0.7 {
                Color32::from_rgb(63, 185, 80) // green
            } else if cov >= 0.4 {
                Color32::from_rgb(210, 153, 34) // yellow
            } else {
                Color32::from_rgb(248, 81, 73) // red
            };
            ui.label(RichText::new(format!("{pct}%")).color(cov_color));
        } else {
            ui.label("—");
        }

        ui.end_row();
    }
}

fn metric_cell(ui: &mut Ui, value: Option<u32>, hue: f64) {
    if let Some(v) = value {
        let color = hue_to_color(hue);
        ui.label(RichText::new(v.to_string()).color(color));
    } else {
        ui.label("—");
    }
}

fn sort_indices(indices: &mut [usize], entries: &[EvalEntry], col: SortColumn, order: SortOrder) {
    indices.sort_by(|&a, &b| {
        let ea = &entries[a];
        let eb = &entries[b];

        let cmp = match col {
            SortColumn::Name => ea.name.cmp(&eb.name),
            SortColumn::Kind => ea.kind.cmp(&eb.kind),
            SortColumn::Lines => ea.lines.cmp(&eb.lines),
            SortColumn::Cyclomatic => ea.cyclomatic.cmp(&eb.cyclomatic),
            SortColumn::Params => ea.params.cmp(&eb.params),
            SortColumn::Depth => ea.depth.cmp(&eb.depth),
            SortColumn::Churn => ea.git_churn_30d.cmp(&eb.git_churn_30d),
            SortColumn::Coverage => ea
                .coverage
                .partial_cmp(&eb.coverage)
                .unwrap_or(std::cmp::Ordering::Equal),
        };

        match order {
            SortOrder::Ascending => cmp,
            SortOrder::Descending => cmp.reverse(),
        }
    });
}
