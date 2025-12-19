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

    /// Get a string describing the kind of the last command (for error messages)
    fn last_cmd_kind(&self) -> &'static str {
        match self.last_cmd() {
            Some(CmdRecord::None) => "None",
            Some(CmdRecord::Task) => "Task",
            Some(CmdRecord::Msg(_)) => "Msg",
            Some(CmdRecord::Batch(_)) => "Batch",
            None => "<no command>",
        }
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

    // ========================================
    // Expect系アサーションメソッド
    // ========================================

    /// Assert that the model satisfies a predicate
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::Inc)
    ///     .expect_model(|m| m.count == 1)
    ///     .send(Msg::Inc)
    ///     .expect_model(|m| m.count == 2);
    /// ```
    ///
    /// # Panics
    /// Panics if the predicate returns false
    pub fn expect_model(&mut self, predicate: impl FnOnce(&A::Model) -> bool) -> &mut Self {
        assert!(
            predicate(&self.model),
            "expect_model: predicate returned false"
        );
        self
    }

    /// Assert that the model satisfies a predicate with custom message
    ///
    /// # Panics
    /// Panics with the provided message if the predicate returns false
    pub fn expect_model_msg(
        &mut self,
        predicate: impl FnOnce(&A::Model) -> bool,
        msg: &str,
    ) -> &mut Self {
        assert!(predicate(&self.model), "expect_model: {}", msg);
        self
    }

    /// Assert that the last command was `Cmd::None`
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::SetValue(42))
    ///     .expect_cmd_none();
    /// ```
    ///
    /// # Panics
    /// Panics if the last command was not `Cmd::None`
    pub fn expect_cmd_none(&mut self) -> &mut Self {
        assert!(
            self.last_was_none(),
            "expect_cmd_none: last command was {}, expected None",
            self.last_cmd_kind()
        );
        self
    }

    /// Assert that the last command was `Cmd::Task`
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::FetchData)
    ///     .expect_cmd_task();
    /// ```
    ///
    /// # Panics
    /// Panics if the last command was not `Cmd::Task`
    pub fn expect_cmd_task(&mut self) -> &mut Self {
        assert!(
            self.last_was_task(),
            "expect_cmd_task: last command was {}, expected Task",
            self.last_cmd_kind()
        );
        self
    }

    /// Assert that the last command was `Cmd::Msg`
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::TriggerDelayed)
    ///     .expect_cmd_msg();
    /// ```
    ///
    /// # Panics
    /// Panics if the last command was not `Cmd::Msg`
    pub fn expect_cmd_msg(&mut self) -> &mut Self {
        assert!(
            self.last_was_msg(),
            "expect_cmd_msg: last command was {}, expected Msg",
            self.last_cmd_kind()
        );
        self
    }

    /// Assert that the last command was `Cmd::Msg` and verify its content
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::TriggerDelayed)
    ///     .expect_cmd_msg_eq(Msg::Inc);
    /// ```
    ///
    /// # Panics
    /// Panics if the last command was not `Cmd::Msg` or the message doesn't match
    pub fn expect_cmd_msg_eq(&mut self, expected: A::Msg) -> &mut Self
    where
        A::Msg: PartialEq + std::fmt::Debug,
    {
        match self.last_cmd() {
            Some(CmdRecord::Msg(msg)) => {
                assert_eq!(
                    msg, &expected,
                    "expect_cmd_msg_eq: message mismatch"
                );
            }
            _ => {
                panic!(
                    "expect_cmd_msg_eq: last command was {}, expected Msg({:?})",
                    self.last_cmd_kind(), expected
                );
            }
        }
        self
    }

    /// Assert that the last command was `Cmd::Batch`
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::MultiAction)
    ///     .expect_cmd_batch();
    /// ```
    ///
    /// # Panics
    /// Panics if the last command was not `Cmd::Batch`
    pub fn expect_cmd_batch(&mut self) -> &mut Self {
        assert!(
            matches!(self.last_cmd(), Some(CmdRecord::Batch(_))),
            "expect_cmd_batch: last command was {}, expected Batch",
            self.last_cmd_kind()
        );
        self
    }

    /// Assert that the last command was `Cmd::Batch` with expected size
    ///
    /// # Example
    /// ```ignore
    /// runner
    ///     .send(Msg::MultiAction)
    ///     .expect_cmd_batch_size(3);
    /// ```
    ///
    /// # Panics
    /// Panics if the last command was not `Cmd::Batch` or size doesn't match
    pub fn expect_cmd_batch_size(&mut self, expected_size: usize) -> &mut Self {
        match self.last_cmd() {
            Some(CmdRecord::Batch(size)) => {
                assert_eq!(
                    *size, expected_size,
                    "expect_cmd_batch_size: batch size mismatch (got {}, expected {})",
                    size, expected_size
                );
            }
            _ => {
                panic!(
                    "expect_cmd_batch_size: last command was {}, expected Batch({})",
                    self.last_cmd_kind(), expected_size
                );
            }
        }
        self
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

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Inc,
        Dec,
        Set(i32),
        Delayed,
        MultiBatch,
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
                TestMsg::MultiBatch => {
                    return Cmd::batch([Cmd::msg(TestMsg::Inc), Cmd::msg(TestMsg::Inc)]);
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

    #[test]
    fn test_expect_model() {
        let mut runner = TestRunner::<TestApp>::new();

        runner
            .send(TestMsg::Inc)
            .expect_model(|m| m.value == 1)
            .send(TestMsg::Inc)
            .expect_model(|m| m.value == 2)
            .send(TestMsg::Set(100))
            .expect_model(|m| m.value == 100);
    }

    #[test]
    fn test_expect_cmd_none() {
        let mut runner = TestRunner::<TestApp>::new();

        runner.send(TestMsg::Inc).expect_cmd_none();
    }

    #[test]
    fn test_expect_cmd_msg() {
        let mut runner = TestRunner::<TestApp>::new();

        runner.send(TestMsg::Delayed).expect_cmd_msg();
    }

    #[test]
    fn test_expect_cmd_msg_eq() {
        let mut runner = TestRunner::<TestApp>::new();

        runner
            .send(TestMsg::Delayed)
            .expect_cmd_msg_eq(TestMsg::Inc);
    }

    #[test]
    fn test_expect_cmd_batch() {
        let mut runner = TestRunner::<TestApp>::new();

        runner
            .send(TestMsg::MultiBatch)
            .expect_cmd_batch()
            .expect_cmd_batch_size(2);
    }

    #[test]
    fn test_expect_chaining() {
        // Fluent API chaining test
        let mut runner = TestRunner::<TestApp>::new();

        runner
            .send(TestMsg::Inc)
            .expect_model(|m| m.value == 1)
            .expect_cmd_none()
            .send(TestMsg::Inc)
            .expect_model(|m| m.value == 2)
            .expect_cmd_none()
            .send(TestMsg::Delayed)
            .expect_model(|m| m.value == 2) // Delayed doesn't change value directly
            .expect_cmd_msg_eq(TestMsg::Inc);
    }
}
