//! Table molecule - themed table with egui_extras
//!
//! Provides Table and DataTable components with theme integration.

use crate::Theme;
use egui::Ui;

#[cfg(feature = "extras")]
use egui_extras::{Column, TableBuilder};

/// A simple table component (without egui_extras)
pub struct Table<'a> {
    headers: &'a [&'a str],
    rows: Vec<Vec<String>>,
    striped: bool,
}

impl<'a> Table<'a> {
    pub fn new(headers: &'a [&'a str]) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            striped: true,
        }
    }

    pub fn row(mut self, cells: Vec<String>) -> Self {
        self.rows.push(cells);
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let theme = Theme::current(ui.ctx());

        egui::Frame::new()
            .stroke(egui::Stroke::new(theme.border_width, theme.border))
            .corner_radius(theme.radius_md)
            .show(ui, |ui| {
                egui::Grid::new(ui.next_auto_id())
                    .num_columns(self.headers.len())
                    .striped(false)
                    .show(ui, |ui| {
                        // Header row
                        for header in self.headers {
                            egui::Frame::new()
                                .fill(theme.bg_secondary)
                                .inner_margin(egui::Margin::symmetric(12, 8))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(*header).strong());
                                });
                        }
                        ui.end_row();

                        // Data rows
                        for (i, row) in self.rows.iter().enumerate() {
                            let bg = if self.striped && i % 2 == 1 {
                                Some(theme.bg_tertiary)
                            } else {
                                None
                            };

                            for cell in row {
                                let mut frame = egui::Frame::new()
                                    .inner_margin(egui::Margin::symmetric(12, 8));

                                if let Some(bg_color) = bg {
                                    frame = frame.fill(bg_color);
                                }

                                frame.show(ui, |ui| {
                                    ui.label(cell);
                                });
                            }
                            ui.end_row();
                        }
                    });
            });
    }
}

/// Builder for table with typed data (uses egui_extras when available)
#[cfg(feature = "extras")]
pub struct DataTable<'a, T> {
    data: &'a [T],
    columns: Vec<DataColumn<'a, T>>,
    striped: bool,
    resizable: bool,
    row_height: Option<f32>,
    selected: Option<usize>,
}

#[cfg(feature = "extras")]
struct DataColumn<'a, T> {
    header: &'a str,
    width: DataColumnWidth,
    accessor: Box<dyn Fn(&T) -> String + 'a>,
}

/// Column width specification
#[cfg(feature = "extras")]
#[derive(Clone, Copy, Default)]
pub enum DataColumnWidth {
    #[default]
    Auto,
    Fixed(f32),
    Initial(f32),
    Remainder,
}

#[cfg(feature = "extras")]
impl<'a, T> DataTable<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self {
            data,
            columns: Vec::new(),
            striped: true,
            resizable: true,
            row_height: None,
            selected: None,
        }
    }

    /// Add a column with auto width
    pub fn column(mut self, header: &'a str, accessor: impl Fn(&T) -> String + 'a) -> Self {
        self.columns.push(DataColumn {
            header,
            width: DataColumnWidth::Auto,
            accessor: Box::new(accessor),
        });
        self
    }

    /// Add a column with specific width
    pub fn column_with_width(
        mut self,
        header: &'a str,
        width: DataColumnWidth,
        accessor: impl Fn(&T) -> String + 'a,
    ) -> Self {
        self.columns.push(DataColumn {
            header,
            width,
            accessor: Box::new(accessor),
        });
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = Some(height);
        self
    }

    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    /// Show the table, returns clicked row index if any
    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        let theme = Theme::current(ui.ctx());
        let row_height = self.row_height.unwrap_or(theme.spacing_lg + theme.spacing_sm);
        let mut clicked_row: Option<usize> = None;

        let mut builder = TableBuilder::new(ui)
            .striped(self.striped)
            .resizable(self.resizable)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        // Add columns
        for col in &self.columns {
            let column = match col.width {
                DataColumnWidth::Auto => Column::auto(),
                DataColumnWidth::Fixed(w) => Column::exact(w),
                DataColumnWidth::Initial(w) => Column::initial(w).resizable(self.resizable),
                DataColumnWidth::Remainder => Column::remainder(),
            };
            builder = builder.column(column);
        }

        builder
            .header(row_height, |mut header| {
                for col in &self.columns {
                    header.col(|ui| {
                        ui.strong(col.header);
                    });
                }
            })
            .body(|mut body| {
                for (idx, item) in self.data.iter().enumerate() {
                    let is_selected = self.selected == Some(idx);

                    body.row(row_height, |mut row| {
                        for col in &self.columns {
                            row.col(|ui| {
                                if is_selected {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        0.0,
                                        theme.primary.gamma_multiply(0.3),
                                    );
                                }

                                let text = (col.accessor)(item);
                                let response = ui.label(&text);

                                if response.clicked() {
                                    clicked_row = Some(idx);
                                }
                            });
                        }
                    });
                }
            });

        clicked_row
    }
}

/// Fallback DataTable without egui_extras
#[cfg(not(feature = "extras"))]
pub struct DataTable<'a, T> {
    data: &'a [T],
    columns: Vec<DataColumnSimple<'a, T>>,
    striped: bool,
}

#[cfg(not(feature = "extras"))]
struct DataColumnSimple<'a, T> {
    header: &'a str,
    accessor: Box<dyn Fn(&T) -> String + 'a>,
}

#[cfg(not(feature = "extras"))]
impl<'a, T> DataTable<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self {
            data,
            columns: Vec::new(),
            striped: true,
        }
    }

    pub fn column(mut self, header: &'a str, accessor: impl Fn(&T) -> String + 'a) -> Self {
        self.columns.push(DataColumnSimple {
            header,
            accessor: Box::new(accessor),
        });
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let headers: Vec<&str> = self.columns.iter().map(|c| c.header).collect();
        let rows: Vec<Vec<String>> = self
            .data
            .iter()
            .map(|item| {
                self.columns
                    .iter()
                    .map(|col| (col.accessor)(item))
                    .collect()
            })
            .collect();

        Table::new(&headers)
            .rows(rows)
            .striped(self.striped)
            .show(ui);
    }
}
