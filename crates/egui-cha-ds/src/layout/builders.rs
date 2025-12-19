//! Layout builders - Col, Row, Grid

use egui::Ui;

/// Trait for items that can be added to layouts
pub trait LayoutItem {
    fn render(self, ui: &mut Ui);
}

/// Blanket implementation for closures
/// Uses HRTB (Higher-Ranked Trait Bounds) for proper lifetime handling
impl<F> LayoutItem for F
where
    F: for<'a> FnOnce(&'a mut Ui),
{
    fn render(self, ui: &mut Ui) {
        self(ui);
    }
}

/// Create a vertical column layout
pub fn col() -> Col {
    Col::new()
}

/// Create a horizontal row layout
pub fn row() -> Row {
    Row::new()
}

/// Create a grid layout with specified columns
pub fn grid(cols: usize) -> Grid {
    Grid::new(cols)
}

// ============================================================
// Col (Vertical Layout)
// ============================================================

/// Vertical column layout builder
pub struct Col {
    spacing: f32,
    items: Vec<Box<dyn FnOnce(&mut Ui)>>,
    fill_x: bool,
    padding: f32,
}

impl Col {
    pub fn new() -> Self {
        Self {
            spacing: 4.0,
            items: Vec::new(),
            fill_x: false,
            padding: 0.0,
        }
    }

    /// Set spacing between items
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Fill available width
    pub fn fill_x(mut self) -> Self {
        self.fill_x = true;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Add an item to the column (accepts closures, Col, Row, Grid, etc.)
    pub fn add<T: LayoutItem + 'static>(mut self, item: T) -> Self {
        self.items.push(Box::new(move |ui: &mut Ui| item.render(ui)));
        self
    }

    /// Add a widget that implements Widget trait
    pub fn add_widget(mut self, widget: impl egui::Widget + 'static) -> Self {
        self.items.push(Box::new(move |ui: &mut Ui| {
            ui.add(widget);
        }));
        self
    }

    /// Show the column layout
    pub fn show(self, ui: &mut Ui) {
        let frame = if self.padding > 0.0 {
            egui::Frame::new().inner_margin(self.padding)
        } else {
            egui::Frame::NONE
        };

        frame.show(ui, |ui| {
            ui.with_layout(
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.spacing_mut().item_spacing.y = self.spacing;

                    if self.fill_x {
                        ui.set_width(ui.available_width());
                    }

                    for item in self.items {
                        item(ui);
                    }
                },
            );
        });
    }
}

impl Default for Col {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutItem for Col {
    fn render(self, ui: &mut Ui) {
        self.show(ui);
    }
}

// ============================================================
// Row (Horizontal Layout)
// ============================================================

/// Horizontal row layout builder
pub struct Row {
    spacing: f32,
    items: Vec<Box<dyn FnOnce(&mut Ui)>>,
    fill_x: bool,
    centered: bool,
    padding: f32,
}

impl Row {
    pub fn new() -> Self {
        Self {
            spacing: 4.0,
            items: Vec::new(),
            fill_x: false,
            centered: false,
            padding: 0.0,
        }
    }

    /// Set spacing between items
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Fill available width
    pub fn fill_x(mut self) -> Self {
        self.fill_x = true;
        self
    }

    /// Center items vertically
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Add an item to the row (accepts closures, Col, Row, Grid, etc.)
    pub fn add<T: LayoutItem + 'static>(mut self, item: T) -> Self {
        self.items.push(Box::new(move |ui: &mut Ui| item.render(ui)));
        self
    }

    /// Add a widget that implements Widget trait
    pub fn add_widget(mut self, widget: impl egui::Widget + 'static) -> Self {
        self.items.push(Box::new(move |ui: &mut Ui| {
            ui.add(widget);
        }));
        self
    }

    /// Show the row layout
    pub fn show(self, ui: &mut Ui) {
        let frame = if self.padding > 0.0 {
            egui::Frame::new().inner_margin(self.padding)
        } else {
            egui::Frame::NONE
        };

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = self.spacing;

                if self.fill_x {
                    ui.set_width(ui.available_width());
                }

                for item in self.items {
                    item(ui);
                }
            });
        });
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutItem for Row {
    fn render(self, ui: &mut Ui) {
        self.show(ui);
    }
}

// ============================================================
// Grid Layout
// ============================================================

/// Grid layout builder
pub struct Grid {
    cols: usize,
    gap: f32,
    items: Vec<Box<dyn FnOnce(&mut Ui)>>,
    padding: f32,
}

impl Grid {
    pub fn new(cols: usize) -> Self {
        Self {
            cols: cols.max(1),
            gap: 8.0,
            items: Vec::new(),
            padding: 0.0,
        }
    }

    /// Set gap between items
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Add an item to the grid (accepts closures, Col, Row, Grid, etc.)
    pub fn add<T: LayoutItem + 'static>(mut self, item: T) -> Self {
        self.items.push(Box::new(move |ui: &mut Ui| item.render(ui)));
        self
    }

    /// Add a widget that implements Widget trait
    pub fn add_widget(mut self, widget: impl egui::Widget + 'static) -> Self {
        self.items.push(Box::new(move |ui: &mut Ui| {
            ui.add(widget);
        }));
        self
    }

    /// Show the grid layout
    pub fn show(self, ui: &mut Ui) {
        let frame = if self.padding > 0.0 {
            egui::Frame::new().inner_margin(self.padding)
        } else {
            egui::Frame::NONE
        };

        frame.show(ui, |ui| {
            egui::Grid::new(ui.next_auto_id())
                .num_columns(self.cols)
                .spacing([self.gap, self.gap])
                .show(ui, |ui| {
                    for (i, item) in self.items.into_iter().enumerate() {
                        item(ui);
                        if (i + 1) % self.cols == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(2)
    }
}

impl LayoutItem for Grid {
    fn render(self, ui: &mut Ui) {
        self.show(ui);
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----------------------------------------------------------
    // Col Tests
    // ----------------------------------------------------------

    #[test]
    fn col_default_values() {
        let col = Col::new();
        assert_eq!(col.spacing, 4.0);
        assert!(!col.fill_x);
        assert_eq!(col.padding, 0.0);
        assert!(col.items.is_empty());
    }

    #[test]
    fn col_spacing() {
        let col = col().spacing(16.0);
        assert_eq!(col.spacing, 16.0);
    }

    #[test]
    fn col_fill_x() {
        let col = col().fill_x();
        assert!(col.fill_x);
    }

    #[test]
    fn col_padding() {
        let col = col().padding(20.0);
        assert_eq!(col.padding, 20.0);
    }

    #[test]
    fn col_add_items() {
        let col = col()
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {});
        assert_eq!(col.items.len(), 3);
    }

    #[test]
    fn col_method_chaining() {
        let col = col()
            .spacing(12.0)
            .fill_x()
            .padding(8.0)
            .add(|_ui: &mut Ui| {});

        assert_eq!(col.spacing, 12.0);
        assert!(col.fill_x);
        assert_eq!(col.padding, 8.0);
        assert_eq!(col.items.len(), 1);
    }

    #[test]
    fn col_default_trait() {
        let col = Col::default();
        assert_eq!(col.spacing, 4.0);
    }

    // ----------------------------------------------------------
    // Row Tests
    // ----------------------------------------------------------

    #[test]
    fn row_default_values() {
        let row = Row::new();
        assert_eq!(row.spacing, 4.0);
        assert!(!row.fill_x);
        assert!(!row.centered);
        assert_eq!(row.padding, 0.0);
        assert!(row.items.is_empty());
    }

    #[test]
    fn row_spacing() {
        let row = row().spacing(24.0);
        assert_eq!(row.spacing, 24.0);
    }

    #[test]
    fn row_fill_x() {
        let row = row().fill_x();
        assert!(row.fill_x);
    }

    #[test]
    fn row_centered() {
        let row = row().centered();
        assert!(row.centered);
    }

    #[test]
    fn row_padding() {
        let row = row().padding(16.0);
        assert_eq!(row.padding, 16.0);
    }

    #[test]
    fn row_add_items() {
        let row = row()
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {});
        assert_eq!(row.items.len(), 2);
    }

    #[test]
    fn row_method_chaining() {
        let row = row()
            .spacing(8.0)
            .fill_x()
            .centered()
            .padding(4.0)
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {});

        assert_eq!(row.spacing, 8.0);
        assert!(row.fill_x);
        assert!(row.centered);
        assert_eq!(row.padding, 4.0);
        assert_eq!(row.items.len(), 2);
    }

    #[test]
    fn row_default_trait() {
        let row = Row::default();
        assert_eq!(row.spacing, 4.0);
    }

    // ----------------------------------------------------------
    // Grid Tests
    // ----------------------------------------------------------

    #[test]
    fn grid_default_values() {
        let grid = Grid::new(3);
        assert_eq!(grid.cols, 3);
        assert_eq!(grid.gap, 8.0);
        assert_eq!(grid.padding, 0.0);
        assert!(grid.items.is_empty());
    }

    #[test]
    fn grid_cols_minimum() {
        // cols should be at least 1
        let grid = Grid::new(0);
        assert_eq!(grid.cols, 1);
    }

    #[test]
    fn grid_gap() {
        let grid = grid(2).gap(16.0);
        assert_eq!(grid.gap, 16.0);
    }

    #[test]
    fn grid_padding() {
        let grid = grid(2).padding(12.0);
        assert_eq!(grid.padding, 12.0);
    }

    #[test]
    fn grid_add_items() {
        let grid = grid(3)
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {})
            .add(|_ui: &mut Ui| {});
        assert_eq!(grid.items.len(), 4);
    }

    #[test]
    fn grid_method_chaining() {
        let grid = grid(4)
            .gap(10.0)
            .padding(5.0)
            .add(|_ui: &mut Ui| {});

        assert_eq!(grid.cols, 4);
        assert_eq!(grid.gap, 10.0);
        assert_eq!(grid.padding, 5.0);
        assert_eq!(grid.items.len(), 1);
    }

    #[test]
    fn grid_default_trait() {
        let grid = Grid::default();
        assert_eq!(grid.cols, 2);
        assert_eq!(grid.gap, 8.0);
    }

    // ----------------------------------------------------------
    // Nested Layout Tests
    // ----------------------------------------------------------

    #[test]
    fn col_can_add_row() {
        // Test that Row can be added to Col directly (via LayoutItem)
        let layout = col()
            .add(row().add(|_ui: &mut Ui| {}));
        assert_eq!(layout.items.len(), 1);
    }

    #[test]
    fn row_can_add_col() {
        // Test that Col can be added to Row directly (via LayoutItem)
        let layout = row()
            .add(col().add(|_ui: &mut Ui| {}));
        assert_eq!(layout.items.len(), 1);
    }

    #[test]
    fn col_can_add_grid() {
        let layout = col()
            .add(grid(3).add(|_ui: &mut Ui| {}));
        assert_eq!(layout.items.len(), 1);
    }

    #[test]
    fn grid_can_add_col() {
        let layout = grid(2)
            .add(col().add(|_ui: &mut Ui| {}))
            .add(col().add(|_ui: &mut Ui| {}));
        assert_eq!(layout.items.len(), 2);
    }

    #[test]
    fn deeply_nested_layouts() {
        // Test deep nesting: Col > Row > Col > Grid
        let layout = col()
            .add(row()
                .add(col()
                    .add(grid(2)
                        .add(|_ui: &mut Ui| {})
                        .add(|_ui: &mut Ui| {}))));
        assert_eq!(layout.items.len(), 1);
    }

    #[test]
    fn complex_nested_structure() {
        // Simulate a dashboard layout
        let layout = col()
            .spacing(16.0)
            .add(row()  // Header
                .fill_x()
                .add(|_ui: &mut Ui| {})  // Logo
                .add(|_ui: &mut Ui| {})) // Nav
            .add(row()  // Main content
                .add(col()  // Sidebar
                    .add(|_ui: &mut Ui| {})
                    .add(|_ui: &mut Ui| {}))
                .add(col()  // Content area
                    .fill_x()
                    .add(grid(3)  // Card grid
                        .gap(8.0)
                        .add(|_ui: &mut Ui| {})
                        .add(|_ui: &mut Ui| {})
                        .add(|_ui: &mut Ui| {}))));

        assert_eq!(layout.items.len(), 2);  // Header + Main
    }

    // ----------------------------------------------------------
    // LayoutItem Trait Tests
    // ----------------------------------------------------------

    #[test]
    fn closure_implements_layout_item() {
        fn accepts_layout_item<T: LayoutItem>(_: T) {}
        accepts_layout_item(|_ui: &mut Ui| {});
    }

    #[test]
    fn col_implements_layout_item() {
        fn accepts_layout_item<T: LayoutItem>(_: T) {}
        accepts_layout_item(col());
    }

    #[test]
    fn row_implements_layout_item() {
        fn accepts_layout_item<T: LayoutItem>(_: T) {}
        accepts_layout_item(row());
    }

    #[test]
    fn grid_implements_layout_item() {
        fn accepts_layout_item<T: LayoutItem>(_: T) {}
        accepts_layout_item(grid(2));
    }

    // ----------------------------------------------------------
    // Integration Tests with egui Context
    // ----------------------------------------------------------

    /// Helper to run UI code in a test context
    fn run_ui_test<F>(f: F)
    where
        F: FnOnce(&mut egui::Ui),
    {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(f) = f.take() {
                    f(ui);
                }
            });
        });
    }

    #[test]
    fn col_renders_without_panic() {
        run_ui_test(|ui| {
            col()
                .spacing(8.0)
                .add(|ui: &mut Ui| {
                    ui.label("Test");
                })
                .show(ui);
        });
    }

    #[test]
    fn row_renders_without_panic() {
        run_ui_test(|ui| {
            row()
                .spacing(8.0)
                .add(|ui: &mut Ui| {
                    ui.label("A");
                })
                .add(|ui: &mut Ui| {
                    ui.label("B");
                })
                .show(ui);
        });
    }

    #[test]
    fn grid_renders_without_panic() {
        run_ui_test(|ui| {
            grid(3)
                .gap(8.0)
                .add(|ui: &mut Ui| { ui.label("1"); })
                .add(|ui: &mut Ui| { ui.label("2"); })
                .add(|ui: &mut Ui| { ui.label("3"); })
                .add(|ui: &mut Ui| { ui.label("4"); })
                .add(|ui: &mut Ui| { ui.label("5"); })
                .add(|ui: &mut Ui| { ui.label("6"); })
                .show(ui);
        });
    }

    #[test]
    fn nested_layout_renders_without_panic() {
        run_ui_test(|ui| {
            col()
                .spacing(16.0)
                .padding(8.0)
                .add(row()
                    .fill_x()
                    .add(|ui: &mut Ui| { ui.label("Header"); }))
                .add(row()
                    .add(col()
                        .add(|ui: &mut Ui| { ui.label("Sidebar 1"); })
                        .add(|ui: &mut Ui| { ui.label("Sidebar 2"); }))
                    .add(col()
                        .fill_x()
                        .add(grid(2)
                            .gap(4.0)
                            .add(|ui: &mut Ui| { ui.label("A"); })
                            .add(|ui: &mut Ui| { ui.label("B"); })
                            .add(|ui: &mut Ui| { ui.label("C"); })
                            .add(|ui: &mut Ui| { ui.label("D"); }))))
                .show(ui);
        });
    }

    #[test]
    fn col_with_padding_renders() {
        run_ui_test(|ui| {
            col()
                .padding(20.0)
                .add(|ui: &mut Ui| { ui.label("Padded content"); })
                .show(ui);
        });
    }

    #[test]
    fn row_centered_renders() {
        run_ui_test(|ui| {
            row()
                .centered()
                .add(|ui: &mut Ui| { ui.label("Centered"); })
                .show(ui);
        });
    }

    #[test]
    fn empty_layouts_render_without_panic() {
        run_ui_test(|ui| {
            col().show(ui);
            row().show(ui);
            grid(3).show(ui);
        });
    }

    #[test]
    fn deeply_nested_renders_without_panic() {
        run_ui_test(|ui| {
            col()
                .add(col()
                    .add(col()
                        .add(col()
                            .add(col()
                                .add(|ui: &mut Ui| { ui.label("Deep"); })))))
                .show(ui);
        });
    }
}
