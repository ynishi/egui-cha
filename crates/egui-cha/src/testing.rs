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

use crate::helpers::Clock;
use crate::{App, Cmd};
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

/// A boxed future for async tasks
type BoxFuture<Msg> = Pin<Box<dyn Future<Output = Msg> + Send + 'static>>;

// ============================================
// FakeClock for testing time-dependent code
// ============================================

/// A fake clock for testing time-dependent code
///
/// Allows manual control of time progression, enabling fast and
/// deterministic tests for `Debouncer`, `Throttler`, etc.
///
/// # Example
/// ```ignore
/// use egui_cha::testing::FakeClock;
/// use egui_cha::helpers::Debouncer;
/// use std::time::Duration;
///
/// let clock = FakeClock::new();
/// let mut debouncer = Debouncer::with_clock(clock.clone());
///
/// debouncer.trigger(Duration::from_millis(500), Msg::Search);
///
/// // Time hasn't passed yet
/// assert!(!debouncer.should_fire());
///
/// // Advance time past the debounce delay
/// clock.advance(Duration::from_millis(600));
/// assert!(debouncer.should_fire());
/// ```
#[derive(Clone)]
pub struct FakeClock {
    current: Rc<Cell<Duration>>,
}

impl FakeClock {
    /// Create a new fake clock starting at time zero
    pub fn new() -> Self {
        Self {
            current: Rc::new(Cell::new(Duration::ZERO)),
        }
    }

    /// Advance the clock by the specified duration
    pub fn advance(&self, duration: Duration) {
        self.current.set(self.current.get() + duration);
    }

    /// Set the clock to a specific time
    pub fn set(&self, time: Duration) {
        self.current.set(time);
    }

    /// Get the current time
    pub fn get(&self) -> Duration {
        self.current.get()
    }

    /// Reset the clock to time zero
    pub fn reset(&self) {
        self.current.set(Duration::ZERO);
    }
}

impl Default for FakeClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Duration {
        self.current.get()
    }
}

/// A test runner for TEA applications
///
/// Provides a convenient way to test update logic without running the UI.
pub struct TestRunner<A: App> {
    model: A::Model,
    commands: Vec<CmdRecord<A::Msg>>,
    pending_tasks: Vec<BoxFuture<A::Msg>>,
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
            pending_tasks: Vec::new(),
        };
        runner.record_cmd(init_cmd);
        runner
    }

    /// Create a test runner with a custom initial model
    pub fn with_model(model: A::Model) -> Self {
        Self {
            model,
            commands: Vec::new(),
            pending_tasks: Vec::new(),
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
            Cmd::Task(future) => {
                self.pending_tasks.push(future);
                CmdRecord::Task
            }
            Cmd::Msg(msg) => CmdRecord::Msg(msg),
            Cmd::Batch(cmds) => {
                let len = cmds.len();
                // Extract tasks from batch
                for cmd in cmds {
                    self.extract_tasks(cmd);
                }
                CmdRecord::Batch(len)
            }
        };
        self.commands.push(record);
    }

    /// Extract tasks from a command (recursively for batches)
    fn extract_tasks(&mut self, cmd: Cmd<A::Msg>) {
        match cmd {
            Cmd::None | Cmd::Msg(_) => {}
            Cmd::Task(future) => {
                self.pending_tasks.push(future);
            }
            Cmd::Batch(cmds) => {
                for cmd in cmds {
                    self.extract_tasks(cmd);
                }
            }
        }
    }

    // ========================================
    // Async task processing
    // ========================================

    /// Get the number of pending async tasks
    pub fn pending_task_count(&self) -> usize {
        self.pending_tasks.len()
    }

    /// Check if there are any pending async tasks
    pub fn has_pending_tasks(&self) -> bool {
        !self.pending_tasks.is_empty()
    }

    /// Process one pending async task
    ///
    /// Executes the first pending task, awaits its result, and sends
    /// the resulting message through update.
    ///
    /// Returns `true` if a task was processed, `false` if no tasks were pending.
    ///
    /// # Example
    /// ```ignore
    /// runner.send(Msg::FetchData);
    /// assert!(runner.has_pending_tasks());
    ///
    /// runner.process_task().await;
    /// assert!(!runner.has_pending_tasks());
    /// ```
    pub async fn process_task(&mut self) -> bool {
        if let Some(task) = self.pending_tasks.pop() {
            let msg = task.await;
            self.send(msg);
            true
        } else {
            false
        }
    }

    /// Process all pending async tasks
    ///
    /// Processes tasks until none remain. Note that processing a task
    /// may add new tasks (if the resulting message produces new Cmd::Task),
    /// so this processes until the queue is fully drained.
    ///
    /// # Example
    /// ```ignore
    /// runner.send(Msg::FetchData);
    /// runner.process_tasks().await;
    /// // All tasks completed, results sent through update
    /// ```
    pub async fn process_tasks(&mut self) -> &mut Self {
        while let Some(task) = self.pending_tasks.pop() {
            let msg = task.await;
            self.send(msg);
        }
        self
    }

    /// Process exactly N pending tasks
    ///
    /// Useful when you want to control the order of task execution
    /// or test intermediate states.
    ///
    /// # Panics
    /// Panics if there are fewer than N pending tasks.
    pub async fn process_n_tasks(&mut self, n: usize) -> &mut Self {
        for i in 0..n {
            assert!(
                !self.pending_tasks.is_empty(),
                "process_n_tasks: expected {} tasks but only {} were available",
                n,
                i
            );
            let task = self.pending_tasks.remove(0);
            let msg = task.await;
            self.send(msg);
        }
        self
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
        AsyncFetch,
        FetchResult(i32),
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
                TestMsg::AsyncFetch => {
                    return Cmd::task(async { TestMsg::FetchResult(42) });
                }
                TestMsg::FetchResult(v) => model.value = v,
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

    #[cfg(feature = "tokio")]
    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_process_task() {
        block_on(async {
            let mut runner = TestRunner::<TestApp>::new();

            // Send a message that produces an async task
            runner.send(TestMsg::AsyncFetch);
            assert!(runner.last_was_task());
            assert!(runner.has_pending_tasks());
            assert_eq!(runner.pending_task_count(), 1);

            // Model hasn't changed yet
            assert_eq!(runner.model().value, 0);

            // Process the async task
            runner.process_task().await;

            // Task completed, result was sent through update
            assert!(!runner.has_pending_tasks());
            assert_eq!(runner.model().value, 42);
        });
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_process_tasks() {
        block_on(async {
            let mut runner = TestRunner::<TestApp>::new();

            // Queue multiple async tasks
            runner.send(TestMsg::AsyncFetch);
            runner.send(TestMsg::AsyncFetch);
            assert_eq!(runner.pending_task_count(), 2);

            // Process all tasks
            runner.process_tasks().await;

            // All tasks completed
            assert!(!runner.has_pending_tasks());
            // Last FetchResult(42) sets value to 42
            assert_eq!(runner.model().value, 42);
        });
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_async_expect_chaining() {
        block_on(async {
            let mut runner = TestRunner::<TestApp>::new();

            runner
                .send(TestMsg::Inc)
                .expect_model(|m| m.value == 1)
                .expect_cmd_none()
                .send(TestMsg::AsyncFetch)
                .expect_cmd_task();

            // Process async task
            runner.process_tasks().await;

            runner.expect_model(|m| m.value == 42);
        });
    }

    // ========================================
    // FakeClock tests
    // ========================================

    #[test]
    fn test_fake_clock_basic() {
        let clock = super::FakeClock::new();

        assert_eq!(clock.get(), Duration::ZERO);

        clock.advance(Duration::from_millis(100));
        assert_eq!(clock.get(), Duration::from_millis(100));

        clock.advance(Duration::from_millis(50));
        assert_eq!(clock.get(), Duration::from_millis(150));
    }

    #[test]
    fn test_fake_clock_set_and_reset() {
        let clock = super::FakeClock::new();

        clock.set(Duration::from_secs(10));
        assert_eq!(clock.get(), Duration::from_secs(10));

        clock.reset();
        assert_eq!(clock.get(), Duration::ZERO);
    }

    #[test]
    fn test_fake_clock_shared() {
        let clock1 = super::FakeClock::new();
        let clock2 = clock1.clone();

        clock1.advance(Duration::from_millis(100));

        // Both clocks share the same time
        assert_eq!(clock2.get(), Duration::from_millis(100));
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_debouncer_with_fake_clock() {
        use crate::helpers::DebouncerWithClock;

        let clock = super::FakeClock::new();
        let mut debouncer = DebouncerWithClock::new(clock.clone());

        // Trigger with 500ms delay
        let _cmd = debouncer.trigger(Duration::from_millis(500), ());
        assert!(debouncer.is_pending());
        assert!(!debouncer.should_fire()); // Not yet

        // Advance 300ms - still not ready
        clock.advance(Duration::from_millis(300));
        assert!(!debouncer.should_fire());

        // Advance another 100ms - still not ready (400ms total)
        clock.advance(Duration::from_millis(100));
        assert!(!debouncer.should_fire());

        // Advance 150ms - now ready (550ms total)
        clock.advance(Duration::from_millis(150));
        assert!(debouncer.should_fire());
        assert!(!debouncer.is_pending());
    }

    #[test]
    #[cfg(feature = "tokio")]
    fn test_debouncer_reset_with_fake_clock() {
        use crate::helpers::DebouncerWithClock;

        let clock = super::FakeClock::new();
        let mut debouncer = DebouncerWithClock::new(clock.clone());

        // First trigger
        let _cmd = debouncer.trigger(Duration::from_millis(500), ());

        // Advance 300ms
        clock.advance(Duration::from_millis(300));
        assert!(!debouncer.should_fire());

        // Trigger again (resets timer)
        let _cmd = debouncer.trigger(Duration::from_millis(500), ());

        // Advance 300ms from reset point - not yet (timer was reset)
        clock.advance(Duration::from_millis(300));
        assert!(!debouncer.should_fire());

        // Advance 250ms more - now ready
        clock.advance(Duration::from_millis(250));
        assert!(debouncer.should_fire());
    }

    // Non-tokio tests using mark_trigger
    #[test]
    fn test_debouncer_with_fake_clock_mark_trigger() {
        use crate::helpers::DebouncerWithClock;

        let clock = super::FakeClock::new();
        let mut debouncer = DebouncerWithClock::new(clock.clone());

        // mark_trigger with 500ms delay
        debouncer.mark_trigger(Duration::from_millis(500));
        assert!(debouncer.is_pending());
        assert!(!debouncer.should_fire());

        // Advance 550ms - now ready
        clock.advance(Duration::from_millis(550));
        assert!(debouncer.should_fire());
        assert!(!debouncer.is_pending());
    }

    #[test]
    fn test_throttler_with_fake_clock() {
        use crate::helpers::ThrottlerWithClock;

        let clock = super::FakeClock::new();
        let mut throttler = ThrottlerWithClock::new(clock.clone());
        let interval = Duration::from_millis(100);

        // First call executes
        let cmd1 = throttler.run(interval, || Cmd::Msg(1));
        assert!(cmd1.is_msg());

        // Immediate second call is throttled
        let cmd2 = throttler.run(interval, || Cmd::Msg(2));
        assert!(cmd2.is_none());

        // Advance 50ms - still throttled
        clock.advance(Duration::from_millis(50));
        let cmd3 = throttler.run(interval, || Cmd::Msg(3));
        assert!(cmd3.is_none());

        // Advance 60ms more (110ms total) - now executes
        clock.advance(Duration::from_millis(60));
        let cmd4 = throttler.run(interval, || Cmd::Msg(4));
        assert!(cmd4.is_msg());
    }

    #[test]
    fn test_throttler_time_remaining_with_fake_clock() {
        use crate::helpers::ThrottlerWithClock;

        let clock = super::FakeClock::new();
        let mut throttler = ThrottlerWithClock::new(clock.clone());
        let interval = Duration::from_millis(100);

        // Before first run
        assert!(throttler.time_remaining(interval).is_none());

        // After first run
        let _ = throttler.run(interval, || Cmd::Msg(1));
        let remaining = throttler.time_remaining(interval);
        assert_eq!(remaining, Some(Duration::from_millis(100)));

        // After 30ms
        clock.advance(Duration::from_millis(30));
        let remaining = throttler.time_remaining(interval);
        assert_eq!(remaining, Some(Duration::from_millis(70)));

        // After interval expires
        clock.advance(Duration::from_millis(80));
        assert!(throttler.time_remaining(interval).is_none());
    }
}
