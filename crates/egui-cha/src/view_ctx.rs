//! ViewCtx - The bridge between view and message emission

use egui::Ui;

/// Context passed to view functions, enabling message emission from any depth
///
/// # Example
/// ```ignore
/// fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
///     if ctx.ui.button("Click me").clicked() {
///         ctx.emit(Msg::ButtonClicked);
///     }
///
///     // Or use the helper
///     ctx.button("Another", Msg::AnotherClicked);
/// }
/// ```
pub struct ViewCtx<'a, Msg> {
    /// The egui UI handle
    pub ui: &'a mut Ui,
    /// Collected messages to be processed after view
    emitter: &'a mut Vec<Msg>,
}

impl<'a, Msg> ViewCtx<'a, Msg> {
    /// Create a new ViewCtx
    pub(crate) fn new(ui: &'a mut Ui, emitter: &'a mut Vec<Msg>) -> Self {
        Self { ui, emitter }
    }

    /// Emit a message to be processed in the next update cycle
    #[inline]
    pub fn emit(&mut self, msg: Msg) {
        self.emitter.push(msg);
    }

    /// Emit multiple messages
    pub fn emit_all(&mut self, msgs: impl IntoIterator<Item = Msg>) {
        self.emitter.extend(msgs);
    }

    /// Emit a Result, converting Ok/Err to appropriate messages
    ///
    /// # Example
    /// ```ignore
    /// ctx.emit_result(
    ///     parse_input(&text),
    ///     |value| Msg::Parsed(value),
    ///     |err| Msg::Error(err.to_string()),
    /// );
    /// ```
    pub fn emit_result<T, E>(
        &mut self,
        result: Result<T, E>,
        on_ok: impl FnOnce(T) -> Msg,
        on_err: impl FnOnce(E) -> Msg,
    ) {
        match result {
            Ok(value) => self.emit(on_ok(value)),
            Err(err) => self.emit(on_err(err)),
        }
    }

    /// Emit only if Result is Err (for error-only handling)
    ///
    /// # Example
    /// ```ignore
    /// ctx.emit_if_err(
    ///     validate(&input),
    ///     |err| Msg::ValidationError(err),
    /// );
    /// ```
    pub fn emit_if_err<T, E>(&mut self, result: Result<T, E>, on_err: impl FnOnce(E) -> Msg) {
        if let Err(err) = result {
            self.emit(on_err(err));
        }
    }

    /// Create a child context for nested UI regions
    ///
    /// Messages from the child are mapped to parent messages via `map_msg`
    pub fn child<'b, ChildMsg, F>(
        &'b mut self,
        child_emitter: &'b mut Vec<ChildMsg>,
        ui: &'b mut Ui,
    ) -> ViewCtx<'b, ChildMsg> {
        ViewCtx {
            ui,
            emitter: child_emitter,
        }
    }

    /// Reborrow with same emitter but different UI (for nested layouts)
    pub fn with_ui<'b>(&'b mut self, ui: &'b mut Ui) -> ViewCtx<'b, Msg>
    where
        'a: 'b,
    {
        ViewCtx {
            ui,
            emitter: self.emitter,
        }
    }
}

// Convenience methods for common patterns
impl<'a, Msg> ViewCtx<'a, Msg> {
    /// Button that emits a message when clicked
    pub fn button(&mut self, text: impl Into<String>, msg: Msg) -> bool {
        let clicked = self.ui.button(text.into()).clicked();
        if clicked {
            self.emit(msg);
        }
        clicked
    }

    /// Horizontal layout
    pub fn horizontal<R>(&mut self, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        let mut child_msgs = Vec::new();
        let result = self
            .ui
            .horizontal(|ui| {
                let mut child_ctx = ViewCtx::new(ui, &mut child_msgs);
                f(&mut child_ctx)
            })
            .inner;
        self.emitter.extend(child_msgs);
        result
    }

    /// Vertical layout
    pub fn vertical<R>(&mut self, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        let mut child_msgs = Vec::new();
        let result = self
            .ui
            .vertical(|ui| {
                let mut child_ctx = ViewCtx::new(ui, &mut child_msgs);
                f(&mut child_ctx)
            })
            .inner;
        self.emitter.extend(child_msgs);
        result
    }

    /// Group (framed region)
    pub fn group<R>(&mut self, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        let mut child_msgs = Vec::new();
        let result = self
            .ui
            .group(|ui| {
                let mut child_ctx = ViewCtx::new(ui, &mut child_msgs);
                f(&mut child_ctx)
            })
            .inner;
        self.emitter.extend(child_msgs);
        result
    }

    /// Collapsing header
    pub fn collapsing<R>(
        &mut self,
        heading: impl Into<String>,
        f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> Option<R> {
        let mut child_msgs = Vec::new();
        let result = self
            .ui
            .collapsing(heading.into(), |ui| {
                let mut child_ctx = ViewCtx::new(ui, &mut child_msgs);
                f(&mut child_ctx)
            })
            .body_returned;
        self.emitter.extend(child_msgs);
        result
    }

    /// Scroll area
    pub fn scroll_area<R>(&mut self, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        let mut child_msgs = Vec::new();
        let result = egui::ScrollArea::vertical()
            .show(self.ui, |ui| {
                let mut child_ctx = ViewCtx::new(ui, &mut child_msgs);
                f(&mut child_ctx)
            })
            .inner;
        self.emitter.extend(child_msgs);
        result
    }

    /// Scroll area with custom id (avoids ID clashes)
    pub fn scroll_area_id<R>(
        &mut self,
        id: impl std::hash::Hash,
        f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        let mut child_msgs = Vec::new();
        let result = egui::ScrollArea::vertical()
            .id_salt(id)
            .show(self.ui, |ui| {
                let mut child_ctx = ViewCtx::new(ui, &mut child_msgs);
                f(&mut child_ctx)
            })
            .inner;
        self.emitter.extend(child_msgs);
        result
    }

    /// Two-panel layout with left sidebar
    ///
    /// Uses egui::SidePanel internally for clean layout.
    ///
    /// # Example
    /// ```ignore
    /// ctx.sidebar_layout(
    ///     "my_sidebar",
    ///     200.0,
    ///     |ctx| {
    ///         // Sidebar content
    ///         ctx.ui.label("Navigation");
    ///     },
    ///     |ctx| {
    ///         // Main content
    ///         ctx.ui.label("Content");
    ///     },
    /// );
    /// ```
    pub fn sidebar_layout(
        &mut self,
        id: impl Into<egui::Id>,
        width: f32,
        sidebar: impl FnOnce(&mut ViewCtx<'_, Msg>),
        main: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        let mut sidebar_msgs = Vec::new();
        let mut main_msgs = Vec::new();
        let egui_ctx = self.ui.ctx().clone();

        // Left sidebar
        egui::SidePanel::left(id)
            .exact_width(width)
            .show(&egui_ctx, |ui| {
                let mut ctx = ViewCtx::new(ui, &mut sidebar_msgs);
                sidebar(&mut ctx);
            });

        // Main panel
        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            let mut ctx = ViewCtx::new(ui, &mut main_msgs);
            main(&mut ctx);
        });

        self.emitter.extend(sidebar_msgs);
        self.emitter.extend(main_msgs);
    }

    /// Two-panel layout with right sidebar
    pub fn sidebar_right_layout(
        &mut self,
        id: impl Into<egui::Id>,
        width: f32,
        main: impl FnOnce(&mut ViewCtx<'_, Msg>),
        sidebar: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        let mut sidebar_msgs = Vec::new();
        let mut main_msgs = Vec::new();
        let egui_ctx = self.ui.ctx().clone();

        // Right sidebar
        egui::SidePanel::right(id)
            .exact_width(width)
            .show(&egui_ctx, |ui| {
                let mut ctx = ViewCtx::new(ui, &mut sidebar_msgs);
                sidebar(&mut ctx);
            });

        // Main panel
        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            let mut ctx = ViewCtx::new(ui, &mut main_msgs);
            main(&mut ctx);
        });

        self.emitter.extend(main_msgs);
        self.emitter.extend(sidebar_msgs);
    }

    /// Top + Main panel layout
    pub fn top_panel_layout(
        &mut self,
        id: impl Into<egui::Id>,
        top: impl FnOnce(&mut ViewCtx<'_, Msg>),
        main: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        let mut top_msgs = Vec::new();
        let mut main_msgs = Vec::new();
        let egui_ctx = self.ui.ctx().clone();

        // Top panel
        egui::TopBottomPanel::top(id).show(&egui_ctx, |ui| {
            let mut ctx = ViewCtx::new(ui, &mut top_msgs);
            top(&mut ctx);
        });

        // Main panel
        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            let mut ctx = ViewCtx::new(ui, &mut main_msgs);
            main(&mut ctx);
        });

        self.emitter.extend(top_msgs);
        self.emitter.extend(main_msgs);
    }

    /// Two-column layout using allocate_ui_at_rect
    ///
    /// Divides the available space into two equal columns.
    /// Each column gets its own ViewCtx with full emit() capability.
    ///
    /// # Example
    /// ```ignore
    /// ctx.two_columns(
    ///     |ctx| {
    ///         ctx.ui.label("Left column");
    ///         ctx.button("Click", Msg::LeftClicked);
    ///     },
    ///     |ctx| {
    ///         ctx.ui.label("Right column");
    ///         ctx.button("Click", Msg::RightClicked);
    ///     },
    /// );
    /// ```
    pub fn two_columns(
        &mut self,
        left: impl FnOnce(&mut ViewCtx<'_, Msg>),
        right: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        self.columns_n::<2>([Box::new(left), Box::new(right)]);
    }

    /// Three-column layout
    ///
    /// Divides the available space into three equal columns.
    pub fn three_columns(
        &mut self,
        col1: impl FnOnce(&mut ViewCtx<'_, Msg>),
        col2: impl FnOnce(&mut ViewCtx<'_, Msg>),
        col3: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        self.columns_n::<3>([Box::new(col1), Box::new(col2), Box::new(col3)]);
    }

    /// Four-column layout
    ///
    /// Divides the available space into four equal columns.
    pub fn four_columns(
        &mut self,
        col1: impl FnOnce(&mut ViewCtx<'_, Msg>),
        col2: impl FnOnce(&mut ViewCtx<'_, Msg>),
        col3: impl FnOnce(&mut ViewCtx<'_, Msg>),
        col4: impl FnOnce(&mut ViewCtx<'_, Msg>),
    ) {
        self.columns_n::<4>([Box::new(col1), Box::new(col2), Box::new(col3), Box::new(col4)]);
    }

    /// Internal helper for N-column layout
    fn columns_n<const N: usize>(
        &mut self,
        columns: [Box<dyn FnOnce(&mut ViewCtx<'_, Msg>) + '_>; N],
    ) {
        let mut all_msgs: Vec<Vec<Msg>> = (0..N).map(|_| Vec::new()).collect();
        let mut columns: Vec<_> = columns.into_iter().map(Some).collect();

        self.ui.columns(N, |cols| {
            for i in 0..N {
                if let Some(col_fn) = columns[i].take() {
                    let mut ctx = ViewCtx::new(&mut cols[i], &mut all_msgs[i]);
                    col_fn(&mut ctx);
                }
            }
        });

        for msgs in all_msgs {
            self.emitter.extend(msgs);
        }
    }
}
