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
}
