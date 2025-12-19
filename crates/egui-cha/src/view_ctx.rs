//! ViewCtx - The bridge between view and message emission

use std::hash::Hash;

use egui::Ui;

use crate::bindings::{ActionBindings, InputBinding};
use crate::drag_drop::{DragSourceResponse, DropZoneResponse};

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

    /// Scroll area (vertical, default settings)
    pub fn scroll_area<R>(&mut self, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        self.scroll_area_with(|area| area, f)
    }

    /// Scroll area with custom id (avoids ID clashes)
    pub fn scroll_area_id<R>(
        &mut self,
        id: impl std::hash::Hash,
        f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        self.scroll_area_with(|area| area.id_salt(id), f)
    }

    /// Scroll area with full customization via builder
    ///
    /// # Example
    /// ```ignore
    /// ctx.scroll_area_with(
    ///     |area| area.max_height(300.0).auto_shrink([false, false]),
    ///     |ctx| {
    ///         for i in 0..100 {
    ///             ctx.ui.label(format!("Item {}", i));
    ///         }
    ///     },
    /// );
    /// ```
    pub fn scroll_area_with<R>(
        &mut self,
        builder: impl FnOnce(egui::ScrollArea) -> egui::ScrollArea,
        f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        let mut child_msgs = Vec::new();
        let area = builder(egui::ScrollArea::vertical());
        let result = area
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

    /// Variable-length column layout
    ///
    /// Divides the available space into N equal columns.
    /// Use this when you need more than 4 columns or dynamic column count.
    ///
    /// # Example
    /// ```ignore
    /// ctx.columns(vec![
    ///     Box::new(|ctx| { ctx.ui.label("Col 1"); }),
    ///     Box::new(|ctx| { ctx.ui.label("Col 2"); }),
    ///     Box::new(|ctx| { ctx.ui.label("Col 3"); }),
    ///     Box::new(|ctx| { ctx.ui.label("Col 4"); }),
    ///     Box::new(|ctx| { ctx.ui.label("Col 5"); }),
    /// ]);
    /// ```
    pub fn columns(&mut self, columns: Vec<Box<dyn FnOnce(&mut ViewCtx<'_, Msg>) + '_>>) {
        let n = columns.len();
        if n == 0 {
            return;
        }

        let mut all_msgs: Vec<Vec<Msg>> = (0..n).map(|_| Vec::new()).collect();
        let mut columns: Vec<_> = columns.into_iter().map(Some).collect();

        self.ui.columns(n, |cols| {
            for i in 0..n {
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

    /// Conditionally show content
    ///
    /// # Example
    /// ```ignore
    /// ctx.show_if(model.is_logged_in, |ctx| {
    ///     ctx.ui.label("Welcome!");
    ///     ctx.button("Logout", Msg::Logout);
    /// });
    /// ```
    pub fn show_if<R>(&mut self, condition: bool, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> Option<R> {
        if condition {
            Some(f(self))
        } else {
            None
        }
    }

    /// Conditionally show content with else branch
    ///
    /// # Example
    /// ```ignore
    /// ctx.show_if_else(
    ///     model.is_logged_in,
    ///     |ctx| { ctx.ui.label("Welcome!"); },
    ///     |ctx| { ctx.button("Login", Msg::Login); },
    /// );
    /// ```
    pub fn show_if_else<R>(
        &mut self,
        condition: bool,
        if_true: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
        if_false: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> R {
        if condition {
            if_true(self)
        } else {
            if_false(self)
        }
    }

    /// Render content with disabled state
    ///
    /// # Example
    /// ```ignore
    /// ctx.enabled_if(model.can_submit, |ctx| {
    ///     ctx.button("Submit", Msg::Submit);
    /// });
    /// ```
    pub fn enabled_if<R>(&mut self, enabled: bool, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        self.ui.add_enabled_ui(enabled, |ui| {
            let mut child_msgs = Vec::new();
            let mut ctx = ViewCtx::new(ui, &mut child_msgs);
            let result = f(&mut ctx);
            self.emitter.extend(child_msgs);
            result
        }).inner
    }

    /// Render content with visible state (hidden but still takes space)
    ///
    /// # Example
    /// ```ignore
    /// ctx.visible_if(model.show_hint, |ctx| {
    ///     ctx.ui.label("This is a hint");
    /// });
    /// ```
    pub fn visible_if<R>(&mut self, visible: bool, f: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R) -> R {
        self.ui.scope(|ui| {
            if !visible {
                ui.set_invisible();
            }
            let mut child_msgs = Vec::new();
            let mut ctx = ViewCtx::new(ui, &mut child_msgs);
            let result = f(&mut ctx);
            self.emitter.extend(child_msgs);
            result
        }).inner
    }
}

// Keyboard shortcuts support
impl<'a, Msg> ViewCtx<'a, Msg> {
    /// Check if a keyboard shortcut was pressed and emit a message
    ///
    /// Uses `consume_shortcut` internally, so the shortcut won't trigger
    /// other handlers after being consumed.
    ///
    /// # Example
    /// ```ignore
    /// use egui::{Key, KeyboardShortcut, Modifiers};
    ///
    /// // Define shortcuts
    /// const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
    /// const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
    ///
    /// // In view function
    /// ctx.on_shortcut(SAVE, Msg::Save);
    /// ctx.on_shortcut(UNDO, Msg::Undo);
    /// ```
    pub fn on_shortcut(&mut self, shortcut: egui::KeyboardShortcut, msg: Msg) -> bool {
        let triggered = self
            .ui
            .ctx()
            .input_mut(|i| i.consume_shortcut(&shortcut));
        if triggered {
            self.emit(msg);
        }
        triggered
    }

    /// Check multiple shortcuts at once
    ///
    /// More efficient than calling `on_shortcut` multiple times.
    ///
    /// # Example
    /// ```ignore
    /// ctx.on_shortcuts(&[
    ///     (SAVE, Msg::Save),
    ///     (UNDO, Msg::Undo),
    ///     (REDO, Msg::Redo),
    /// ]);
    /// ```
    pub fn on_shortcuts(&mut self, shortcuts: &[(egui::KeyboardShortcut, Msg)])
    where
        Msg: Clone,
    {
        for (shortcut, msg) in shortcuts {
            self.on_shortcut(*shortcut, msg.clone());
        }
    }

    /// Check if a specific key was pressed (without modifiers)
    ///
    /// # Example
    /// ```ignore
    /// ctx.on_key(Key::Escape, Msg::Cancel);
    /// ctx.on_key(Key::Enter, Msg::Confirm);
    /// ```
    pub fn on_key(&mut self, key: egui::Key, msg: Msg) -> bool {
        let pressed = self.ui.ctx().input(|i| i.key_pressed(key));
        if pressed {
            self.emit(msg);
        }
        pressed
    }

    /// Check if an input binding was triggered and emit a message.
    ///
    /// Works with any type implementing `InputBinding`, including
    /// `KeyboardShortcut`, `DynamicShortcut`, and `ShortcutGroup`.
    ///
    /// # Example
    /// ```ignore
    /// use egui_cha::bindings::{DynamicShortcut, InputBinding};
    ///
    /// let custom = DynamicShortcut::new(Modifiers::CTRL, Key::K);
    /// ctx.on_binding(&custom, Msg::Custom);
    ///
    /// // Also works with static shortcuts
    /// ctx.on_binding(&shortcuts::SAVE, Msg::Save);
    /// ```
    pub fn on_binding(&mut self, binding: &impl InputBinding, msg: Msg) -> bool {
        let ctx = self.ui.ctx().clone();
        if binding.consume(&ctx) {
            self.emit(msg);
            true
        } else {
            false
        }
    }

    /// Check if an action from ActionBindings was triggered.
    ///
    /// This is the preferred way to handle keyboard shortcuts when
    /// using the dynamic binding system.
    ///
    /// # Example
    /// ```ignore
    /// use egui_cha::bindings::ActionBindings;
    ///
    /// #[derive(Clone, PartialEq, Eq, Hash)]
    /// enum Action { Save, Undo, Redo }
    ///
    /// let bindings = ActionBindings::new()
    ///     .with_default(Action::Save, shortcuts::SAVE)
    ///     .with_default(Action::Undo, shortcuts::UNDO);
    ///
    /// // In view function
    /// ctx.on_action(&bindings, &Action::Save, Msg::Save);
    /// ctx.on_action(&bindings, &Action::Undo, Msg::Undo);
    /// ```
    pub fn on_action<A>(&mut self, bindings: &ActionBindings<A>, action: &A, msg: Msg) -> bool
    where
        A: Eq + Hash + Clone,
    {
        if let Some(shortcut) = bindings.get(action) {
            self.on_binding(shortcut, msg)
        } else {
            false
        }
    }

    /// Check all actions in ActionBindings and return triggered ones.
    ///
    /// Useful when you want to handle multiple actions in a single call.
    ///
    /// # Example
    /// ```ignore
    /// for action in ctx.triggered_actions(&bindings) {
    ///     match action {
    ///         Action::Save => ctx.emit(Msg::Save),
    ///         Action::Undo => ctx.emit(Msg::Undo),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn triggered_actions<A>(&mut self, bindings: &ActionBindings<A>) -> Option<A>
    where
        A: Eq + Hash + Clone,
    {
        let ctx = self.ui.ctx().clone();
        bindings.check_triggered(&ctx).cloned()
    }
}

// Drag & Drop support
impl<'a, Msg> ViewCtx<'a, Msg> {
    /// Create a draggable source
    ///
    /// # Example
    /// ```ignore
    /// ctx.drag_source("item_1", item.clone(), |ctx| {
    ///     ctx.ui.label(&item.name);
    /// }).on_drag_start(ctx, Msg::DragStart { id: item.id });
    /// ```
    pub fn drag_source<P, R>(
        &mut self,
        id: impl Into<egui::Id>,
        payload: P,
        content: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> DragSourceResponse<R>
    where
        P: Clone + Send + Sync + 'static,
    {
        let id = id.into();
        let mut child_msgs = Vec::new();
        let mut inner_result = None;
        let mut drag_started = false;

        let response = self
            .ui
            .dnd_drag_source(id, payload, |ui| {
                let mut ctx = ViewCtx::new(ui, &mut child_msgs);
                inner_result = Some(content(&mut ctx));
            })
            .response;

        // Check if drag started this frame
        if response.drag_started() {
            drag_started = true;
        }

        self.emitter.extend(child_msgs);

        DragSourceResponse {
            inner: inner_result.expect("content closure should have been called"),
            response,
            drag_started,
        }
    }

    /// Create a drop zone that accepts payloads of type P
    ///
    /// # Example
    /// ```ignore
    /// ctx.drop_zone::<Item, _>(|ctx| {
    ///     ctx.ui.label("Drop items here");
    /// }).on_drop(ctx, |item| Msg::ItemDropped(item));
    /// ```
    pub fn drop_zone<P, R>(
        &mut self,
        content: impl FnOnce(&mut ViewCtx<'_, Msg>) -> R,
    ) -> DropZoneResponse<P, R>
    where
        P: Clone + Send + Sync + 'static,
    {
        let mut child_msgs = Vec::new();

        let (response, dropped_payload) = self.ui.dnd_drop_zone::<P, _>(egui::Frame::default(), |ui| {
            let mut ctx = ViewCtx::new(ui, &mut child_msgs);
            content(&mut ctx)
        });

        // Check if being dragged over with compatible payload
        let is_being_dragged_over = egui::DragAndDrop::has_payload_of_type::<P>(self.ui.ctx());

        self.emitter.extend(child_msgs);

        DropZoneResponse {
            inner: response.inner,
            response: response.response,
            payload: dropped_payload,
            is_being_dragged_over,
        }
    }
}
