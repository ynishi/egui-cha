//! Table molecule

use egui::{Color32, RichText, Ui};

/// A simple table component
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
        let is_dark = ui.ctx().style().visuals.dark_mode;

        let header_bg = if is_dark {
            Color32::from_rgb(55, 65, 81)
        } else {
            Color32::from_rgb(243, 244, 246)
        };

        let stripe_bg = if is_dark {
            Color32::from_rgb(31, 41, 55)
        } else {
            Color32::from_rgb(249, 250, 251)
        };

        let border = if is_dark {
            Color32::from_rgb(75, 85, 99)
        } else {
            Color32::from_rgb(229, 231, 235)
        };

        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, border))
            .corner_radius(6.0)
            .show(ui, |ui| {
                egui::Grid::new(ui.next_auto_id())
                    .num_columns(self.headers.len())
                    .striped(false) // We'll handle striping manually
                    .show(ui, |ui| {
                        // Header row
                        for header in self.headers {
                            egui::Frame::new()
                                .fill(header_bg)
                                .inner_margin(egui::Margin::symmetric(12, 8))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(*header).strong());
                                });
                        }
                        ui.end_row();

                        // Data rows
                        for (i, row) in self.rows.iter().enumerate() {
                            let bg = if self.striped && i % 2 == 1 {
                                Some(stripe_bg)
                            } else {
                                None
                            };

                            for cell in row {
                                let mut frame =
                                    egui::Frame::new().inner_margin(egui::Margin::symmetric(12, 8));

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

/// Builder for table with typed data
pub struct DataTable<'a, T> {
    data: &'a [T],
    columns: Vec<Column<'a, T>>,
    striped: bool,
}

pub struct Column<'a, T> {
    header: &'a str,
    accessor: Box<dyn Fn(&T) -> String + 'a>,
}

impl<'a, T> DataTable<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        Self {
            data,
            columns: Vec::new(),
            striped: true,
        }
    }

    pub fn column(mut self, header: &'a str, accessor: impl Fn(&T) -> String + 'a) -> Self {
        self.columns.push(Column {
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
