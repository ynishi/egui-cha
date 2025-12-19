//! Testing utilities for egui-cha applications
//!
//! # Example
//! ```ignore
//! use egui_cha::testing::TestRunner;
//!
//! #[test]
//! fn test_counter_flow() {
//!     let mut runner = TestRunner::<CounterApp>::new();
//!
//!     runner
//!         .send(Msg::Increment)
//!         .send(Msg::Increment)
//!         .send(Msg::Decrement);
//!
//!     assert_eq!(runner.model().count, 1);
//! }
//! ```

use crate::{App, Cmd};

/// A test runner for TEA applications
///
/// Provides a convenient way to test update logic without running the UI.
pub struct TestRunner<A: App> {
    model: A::Model,
    commands: Vec<CmdRecord<A::Msg>>,
}

/// Record of a command that was returned from update
#[derive(Debug)]
pub enum CmdRecord<Msg> {
    None,
    Task,
    Msg(Msg),
    Batch(usize),
}

impl<A: App> TestRunner<A> {
    /// Create a new test runner with initial model
    pub fn new() -> Self {
        let (model, init_cmd) = A::init();
        let mut runner = Self {
            model,
            commands: Vec::new(),
        };
        runner.record_cmd(init_cmd);
        runner
    }

    /// Create a test runner with a custom initial model
    pub fn with_model(model: A::Model) -> Self {
        Self {
            model,
            commands: Vec::new(),
        }
    }

    /// Send a message and process the update
    pub fn send(&mut self, msg: A::Msg) -> &mut Self {
        let cmd = A::update(&mut self.model, msg);
        self.record_cmd(cmd);
        self
    }

    /// Send multiple messages in sequence
    pub fn send_all(&mut self, msgs: impl IntoIterator<Item = A::Msg>) -> &mut Self {
        for msg in msgs {
            self.send(msg);
        }
        self
    }

    /// Get a reference to the current model
    pub fn model(&self) -> &A::Model {
        &self.model
    }

    /// Get a mutable reference to the model (for setup)
    pub fn model_mut(&mut self) -> &mut A::Model {
        &mut self.model
    }

    /// Get the last command record
    pub fn last_cmd(&self) -> Option<&CmdRecord<A::Msg>> {
        self.commands.last()
    }

    /// Get all command records
    pub fn commands(&self) -> &[CmdRecord<A::Msg>] {
        &self.commands
    }

    /// Clear command history
    pub fn clear_commands(&mut self) -> &mut Self {
        self.commands.clear();
        self
    }

    /// Check if the last command was Cmd::None
    pub fn last_was_none(&self) -> bool {
        matches!(self.last_cmd(), Some(CmdRecord::None))
    }

    /// Check if the last command was Cmd::Task
    pub fn last_was_task(&self) -> bool {
        matches!(self.last_cmd(), Some(CmdRecord::Task))
    }

    /// Check if the last command was Cmd::Msg
    pub fn last_was_msg(&self) -> bool {
        matches!(self.last_cmd(), Some(CmdRecord::Msg(_)))
    }

    fn record_cmd(&mut self, cmd: Cmd<A::Msg>) {
        let record = match cmd {
            Cmd::None => CmdRecord::None,
            Cmd::Task(_) => CmdRecord::Task,
            Cmd::Msg(msg) => CmdRecord::Msg(msg),
            Cmd::Batch(cmds) => CmdRecord::Batch(cmds.len()),
        };
        self.commands.push(record);
    }
}

impl<A: App> Default for TestRunner<A> {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for asserting on model state
pub trait ModelAssert<T> {
    /// Assert with a predicate
    fn assert_that(&self, predicate: impl FnOnce(&T) -> bool, msg: &str);
}

impl<A: App> ModelAssert<A::Model> for TestRunner<A> {
    fn assert_that(&self, predicate: impl FnOnce(&A::Model) -> bool, msg: &str) {
        assert!(predicate(&self.model), "{}", msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test app for testing the test runner
    struct TestApp;

    #[derive(Default)]
    struct TestModel {
        value: i32,
    }

    #[derive(Clone, Debug)]
    enum TestMsg {
        Inc,
        Dec,
        Set(i32),
        Delayed,
    }

    impl App for TestApp {
        type Model = TestModel;
        type Msg = TestMsg;

        fn init() -> (Self::Model, Cmd<Self::Msg>) {
            (TestModel::default(), Cmd::none())
        }

        fn update(model: &mut Self::Model, msg: Self::Msg) -> Cmd<Self::Msg> {
            match msg {
                TestMsg::Inc => model.value += 1,
                TestMsg::Dec => model.value -= 1,
                TestMsg::Set(v) => model.value = v,
                TestMsg::Delayed => {
                    return Cmd::msg(TestMsg::Inc);
                }
            }
            Cmd::none()
        }

        fn view(_model: &Self::Model, _ctx: &mut crate::ViewCtx<Self::Msg>) {
            // No-op for testing
        }
    }

    #[test]
    fn test_runner_basic() {
        let mut runner = TestRunner::<TestApp>::new();

        runner.send(TestMsg::Inc);
        assert_eq!(runner.model().value, 1);

        runner.send(TestMsg::Inc).send(TestMsg::Inc);
        assert_eq!(runner.model().value, 3);

        runner.send(TestMsg::Dec);
        assert_eq!(runner.model().value, 2);
    }

    #[test]
    fn test_runner_cmd_tracking() {
        let mut runner = TestRunner::<TestApp>::new();

        runner.send(TestMsg::Inc);
        assert!(runner.last_was_none());

        runner.send(TestMsg::Delayed);
        assert!(runner.last_was_msg());
    }

    #[test]
    fn test_runner_send_all() {
        let mut runner = TestRunner::<TestApp>::new();

        runner.send_all([TestMsg::Inc, TestMsg::Inc, TestMsg::Inc]);
        assert_eq!(runner.model().value, 3);
    }
}
