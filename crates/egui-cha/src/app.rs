//! Core App trait - The heart of TEA

use crate::{sub::Sub, Cmd, ViewCtx};

/// The main application trait following TEA (The Elm Architecture)
///
/// # Type Parameters
/// - `Model`: Application state
/// - `Msg`: Message type for state updates
///
/// # Example
/// ```ignore
/// struct MyApp;
///
/// impl App for MyApp {
///     type Model = AppModel;
///     type Msg = AppMsg;
///
///     fn init() -> (Self::Model, Cmd<Self::Msg>) {
///         (AppModel::default(), Cmd::none())
///     }
///
///     fn update(model: &mut Self::Model, msg: Self::Msg) -> Cmd<Self::Msg> {
///         match msg {
///             AppMsg::Increment => model.count += 1,
///             AppMsg::Decrement => model.count -= 1,
///         }
///         Cmd::none()
///     }
///
///     fn view(model: &Self::Model, ctx: &mut ViewCtx<Self::Msg>) {
///         ctx.ui.label(format!("Count: {}", model.count));
///     }
/// }
/// ```
pub trait App: Sized + 'static {
    /// Application state
    type Model: Send + 'static;

    /// Message type for triggering state updates
    type Msg: Clone + Send + 'static;

    /// Initialize the application with initial model and optional commands
    fn init() -> (Self::Model, Cmd<Self::Msg>);

    /// Update the model based on a message, optionally returning commands
    fn update(model: &mut Self::Model, msg: Self::Msg) -> Cmd<Self::Msg>;

    /// Render the view - use `ctx.emit()` to dispatch messages
    fn view(model: &Self::Model, ctx: &mut ViewCtx<Self::Msg>);

    /// Declare subscriptions based on current model state
    ///
    /// Called each frame. The runtime manages starting/stopping
    /// subscriptions as they appear or disappear.
    ///
    /// # Example
    /// ```ignore
    /// fn subscriptions(model: &Model) -> Sub<Msg> {
    ///     if model.auto_refresh {
    ///         Sub::interval("refresh", Duration::from_secs(30), Msg::Refresh)
    ///     } else {
    ///         Sub::none()
    ///     }
    /// }
    /// ```
    fn subscriptions(_model: &Self::Model) -> Sub<Self::Msg> {
        Sub::none()
    }
}
