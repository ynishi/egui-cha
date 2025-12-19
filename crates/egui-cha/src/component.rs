//! Reusable Component trait

use crate::{Cmd, ViewCtx};

/// A reusable UI component with its own state and messages
///
/// Components follow a "parent owns state" model (Elm-style).
/// The parent allocates and stores the component's State,
/// and passes Props when rendering.
///
/// # Example
/// ```ignore
/// struct Counter;
///
/// impl Component for Counter {
///     type Props = i32;  // initial value
///     type State = i32;  // current count
///     type Msg = CounterMsg;
///
///     fn init(props: &Self::Props) -> Self::State {
///         *props
///     }
///
///     fn update(state: &mut Self::State, msg: Self::Msg) -> Cmd<Self::Msg> {
///         match msg {
///             CounterMsg::Increment => *state += 1,
///             CounterMsg::Decrement => *state -= 1,
///         }
///         Cmd::none()
///     }
///
///     fn view(props: &Self::Props, state: &Self::State, ctx: &mut ViewCtx<Self::Msg>) {
///         ctx.horizontal(|ctx| {
///             ctx.button("-", CounterMsg::Decrement);
///             ctx.ui.label(format!("{}", state));
///             ctx.button("+", CounterMsg::Increment);
///         });
///     }
/// }
/// ```
pub trait Component: Sized {
    /// Props passed from parent (immutable, for display/config)
    type Props;

    /// Internal state (owned by parent, mutated via update)
    type State;

    /// Messages for this component
    type Msg: Clone + Send + 'static;

    /// Initialize state from props
    fn init(props: &Self::Props) -> Self::State;

    /// Update state based on message
    fn update(state: &mut Self::State, msg: Self::Msg) -> Cmd<Self::Msg>;

    /// Render the component
    fn view(props: &Self::Props, state: &Self::State, ctx: &mut ViewCtx<Self::Msg>);
}

/// Extension trait for ViewCtx to mount components
impl<'a, ParentMsg> ViewCtx<'a, ParentMsg> {
    /// Mount a child component, mapping its messages to parent messages
    ///
    /// # Example
    /// ```ignore
    /// ctx.mount::<Counter>(
    ///     &counter_props,
    ///     &counter_state,
    ///     |counter_msg| AppMsg::Counter(counter_msg),
    /// );
    /// ```
    pub fn mount<C>(
        &mut self,
        props: &C::Props,
        state: &C::State,
        map_msg: impl Fn(C::Msg) -> ParentMsg,
    ) where
        C: Component,
    {
        let mut child_msgs: Vec<C::Msg> = Vec::new();

        // Create child context with child's message type
        {
            let mut child_ctx = ViewCtx::new(self.ui, &mut child_msgs);
            C::view(props, state, &mut child_ctx);
        }

        // Map child messages to parent messages
        for child_msg in child_msgs {
            self.emit(map_msg(child_msg));
        }
    }

    /// Mount a child component with mutable state access
    /// Use this when you need to update child state immediately
    pub fn mount_mut<C>(
        &mut self,
        props: &C::Props,
        state: &mut C::State,
        map_msg: impl Fn(C::Msg) -> ParentMsg,
    ) where
        C: Component,
    {
        let mut child_msgs: Vec<C::Msg> = Vec::new();

        {
            let mut child_ctx = ViewCtx::new(self.ui, &mut child_msgs);
            C::view(props, state, &mut child_ctx);
        }

        // Process child messages immediately on child state
        for child_msg in child_msgs {
            let _cmd = C::update(state, child_msg.clone());
            // Note: We could process Cmd here, but for now we just emit to parent
            self.emit(map_msg(child_msg));
        }
    }
}
