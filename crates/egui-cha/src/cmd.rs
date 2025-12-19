//! Command type for side effects (async tasks, HTTP, timers, etc.)

use std::future::Future;
use std::pin::Pin;

/// A command representing a side effect to be executed
///
/// Commands are declarative descriptions of side effects. The runtime
/// executes them and feeds resulting messages back into the update loop.
pub enum Cmd<Msg> {
    /// No side effect
    None,

    /// Multiple commands to execute
    Batch(Vec<Cmd<Msg>>),

    /// An async task that produces a message
    Task(Pin<Box<dyn Future<Output = Msg> + Send + 'static>>),

    /// Emit a message immediately (next frame)
    Msg(Msg),
}

impl<Msg> Cmd<Msg> {
    /// Create an empty command (no side effect)
    #[inline]
    pub fn none() -> Self {
        Cmd::None
    }

    /// Create a batch of commands
    pub fn batch(cmds: impl IntoIterator<Item = Cmd<Msg>>) -> Self {
        let cmds: Vec<_> = cmds.into_iter().collect();
        if cmds.is_empty() {
            Cmd::None
        } else if cmds.len() == 1 {
            cmds.into_iter().next().unwrap()
        } else {
            Cmd::Batch(cmds)
        }
    }

    /// Create a command from an async task
    pub fn task<F>(future: F) -> Self
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        Cmd::Task(Box::pin(future))
    }

    /// Create a command that emits a message immediately
    pub fn msg(msg: Msg) -> Self {
        Cmd::Msg(msg)
    }

    /// Map the message type
    pub fn map<F, NewMsg>(self, f: F) -> Cmd<NewMsg>
    where
        F: Fn(Msg) -> NewMsg + Send + Sync + Clone + 'static,
        Msg: Send + 'static,
        NewMsg: Send + 'static,
    {
        match self {
            Cmd::None => Cmd::None,
            Cmd::Batch(cmds) => Cmd::Batch(cmds.into_iter().map(|c| c.map(f.clone())).collect()),
            Cmd::Task(fut) => {
                let f = f.clone();
                Cmd::Task(Box::pin(async move { f(fut.await) }))
            }
            Cmd::Msg(msg) => Cmd::Msg(f(msg)),
        }
    }
}

impl<Msg> Default for Cmd<Msg> {
    fn default() -> Self {
        Cmd::None
    }
}

// ============================================================
// Test helpers
// ============================================================

impl<Msg> Cmd<Msg> {
    /// Check if this is Cmd::None
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Cmd::None)
    }

    /// Check if this is Cmd::Task
    #[inline]
    pub fn is_task(&self) -> bool {
        matches!(self, Cmd::Task(_))
    }

    /// Check if this is Cmd::Msg
    #[inline]
    pub fn is_msg(&self) -> bool {
        matches!(self, Cmd::Msg(_))
    }

    /// Check if this is Cmd::Batch
    #[inline]
    pub fn is_batch(&self) -> bool {
        matches!(self, Cmd::Batch(_))
    }

    /// Get the message if this is Cmd::Msg, panics otherwise
    ///
    /// # Panics
    /// Panics if the command is not Cmd::Msg
    pub fn unwrap_msg(self) -> Msg {
        match self {
            Cmd::Msg(msg) => msg,
            Cmd::None => panic!("called unwrap_msg on Cmd::None"),
            Cmd::Task(_) => panic!("called unwrap_msg on Cmd::Task"),
            Cmd::Batch(_) => panic!("called unwrap_msg on Cmd::Batch"),
        }
    }

    /// Get the message if this is Cmd::Msg
    pub fn as_msg(&self) -> Option<&Msg> {
        match self {
            Cmd::Msg(msg) => Some(msg),
            _ => None,
        }
    }

    /// Get the batch if this is Cmd::Batch
    pub fn as_batch(&self) -> Option<&[Cmd<Msg>]> {
        match self {
            Cmd::Batch(cmds) => Some(cmds),
            _ => None,
        }
    }

    /// Get the number of commands (1 for non-batch, n for batch, 0 for none)
    pub fn len(&self) -> usize {
        match self {
            Cmd::None => 0,
            Cmd::Batch(cmds) => cmds.len(),
            _ => 1,
        }
    }

    /// Check if empty (only true for Cmd::None)
    pub fn is_empty(&self) -> bool {
        matches!(self, Cmd::None)
    }
}

impl<Msg: PartialEq> Cmd<Msg> {
    /// Check if this is Cmd::Msg with a specific message
    pub fn is_msg_eq(&self, expected: &Msg) -> bool {
        match self {
            Cmd::Msg(msg) => msg == expected,
            _ => false,
        }
    }
}

impl<Msg: std::fmt::Debug> Cmd<Msg> {
    /// Assert this is Cmd::None, panics with debug info otherwise
    #[track_caller]
    pub fn assert_none(&self) {
        assert!(
            self.is_none(),
            "expected Cmd::None, got {:?}",
            self.variant_name()
        );
    }

    /// Assert this is Cmd::Task, panics with debug info otherwise
    #[track_caller]
    pub fn assert_task(&self) {
        assert!(
            self.is_task(),
            "expected Cmd::Task, got {:?}",
            self.variant_name()
        );
    }

    /// Assert this is Cmd::Msg, panics with debug info otherwise
    #[track_caller]
    pub fn assert_msg(&self) {
        assert!(
            self.is_msg(),
            "expected Cmd::Msg, got {:?}",
            self.variant_name()
        );
    }

    fn variant_name(&self) -> &'static str {
        match self {
            Cmd::None => "Cmd::None",
            Cmd::Task(_) => "Cmd::Task",
            Cmd::Msg(_) => "Cmd::Msg",
            Cmd::Batch(_) => "Cmd::Batch",
        }
    }
}

// Utility functions for common patterns
impl<Msg: Send + 'static> Cmd<Msg> {
    /// Create a delayed message (timer)
    pub fn delay(duration: std::time::Duration, msg: Msg) -> Self
    where
        Msg: Clone,
    {
        Cmd::task(async move {
            tokio::time::sleep(duration).await;
            msg
        })
    }

    /// Create a command from an async task that returns Result
    ///
    /// # Example
    /// ```ignore
    /// Cmd::try_task(
    ///     async { fetch_user(id).await },
    ///     |user| Msg::UserLoaded(user),
    ///     |err| Msg::Error(err.to_string()),
    /// )
    /// ```
    pub fn try_task<F, T, E, FnOk, FnErr>(future: F, on_ok: FnOk, on_err: FnErr) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        FnOk: FnOnce(T) -> Msg + Send + 'static,
        FnErr: FnOnce(E) -> Msg + Send + 'static,
    {
        Cmd::task(async move {
            match future.await {
                Ok(value) => on_ok(value),
                Err(err) => on_err(err),
            }
        })
    }

    /// Create a command from a Result, converting to Msg immediately
    ///
    /// # Example
    /// ```ignore
    /// Cmd::from_result(
    ///     parse_config(),
    ///     |config| Msg::ConfigLoaded(config),
    ///     |err| Msg::Error(err.to_string()),
    /// )
    /// ```
    pub fn from_result<T, E, FnOk, FnErr>(result: Result<T, E>, on_ok: FnOk, on_err: FnErr) -> Self
    where
        FnOk: FnOnce(T) -> Msg,
        FnErr: FnOnce(E) -> Msg,
    {
        match result {
            Ok(value) => Cmd::Msg(on_ok(value)),
            Err(err) => Cmd::Msg(on_err(err)),
        }
    }

    /// Create an async task with a timeout
    ///
    /// If the task completes before the timeout, `on_ok` is called with the result.
    /// If the timeout expires first, `on_timeout` is returned.
    ///
    /// # Example
    /// ```ignore
    /// Cmd::with_timeout(
    ///     Duration::from_secs(5),
    ///     fetch_user(user_id),
    ///     |user| Msg::UserLoaded(user),
    ///     Msg::FetchTimeout,
    /// )
    /// ```
    pub fn with_timeout<F, T>(
        timeout: std::time::Duration,
        future: F,
        on_ok: impl FnOnce(T) -> Msg + Send + 'static,
        on_timeout: Msg,
    ) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Cmd::task(async move {
            match tokio::time::timeout(timeout, future).await {
                Ok(value) => on_ok(value),
                Err(_elapsed) => on_timeout,
            }
        })
    }

    /// Create an async task with timeout that returns Result
    ///
    /// Combines `try_task` with timeout handling.
    ///
    /// # Example
    /// ```ignore
    /// Cmd::try_with_timeout(
    ///     Duration::from_secs(5),
    ///     api::fetch_data(),
    ///     |data| Msg::DataLoaded(data),
    ///     |err| Msg::FetchError(err.to_string()),
    ///     Msg::FetchTimeout,
    /// )
    /// ```
    pub fn try_with_timeout<F, T, E, FnOk, FnErr>(
        timeout: std::time::Duration,
        future: F,
        on_ok: FnOk,
        on_err: FnErr,
        on_timeout: Msg,
    ) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        FnOk: FnOnce(T) -> Msg + Send + 'static,
        FnErr: FnOnce(E) -> Msg + Send + 'static,
    {
        Cmd::task(async move {
            match tokio::time::timeout(timeout, future).await {
                Ok(Ok(value)) => on_ok(value),
                Ok(Err(err)) => on_err(err),
                Err(_elapsed) => on_timeout,
            }
        })
    }

    /// Retry an async task with exponential backoff
    ///
    /// Attempts the task up to `max_attempts` times. On failure, waits with
    /// exponential backoff (doubling the delay each time) before retrying.
    ///
    /// # Arguments
    /// - `max_attempts`: Maximum number of attempts (must be >= 1)
    /// - `initial_delay`: Delay before first retry (doubles each retry)
    /// - `make_future`: Factory function that creates the future for each attempt
    /// - `on_ok`: Called with the successful result
    /// - `on_fail`: Called with the last error and total attempt count
    ///
    /// # Example
    /// ```ignore
    /// Cmd::retry(
    ///     3, // max 3 attempts
    ///     Duration::from_millis(100), // start with 100ms delay
    ///     || api::fetch_data(),
    ///     |data| Msg::DataLoaded(data),
    ///     |err, attempts| Msg::FetchFailed(err.to_string(), attempts),
    /// )
    /// ```
    ///
    /// With 3 attempts and 100ms initial delay:
    /// - Attempt 1: immediate
    /// - Attempt 2: after 100ms (if attempt 1 failed)
    /// - Attempt 3: after 200ms (if attempt 2 failed)
    pub fn retry<F, Fut, T, E, FnOk, FnFail>(
        max_attempts: u32,
        initial_delay: std::time::Duration,
        make_future: F,
        on_ok: FnOk,
        on_fail: FnFail,
    ) -> Self
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send + 'static,
        E: Send + 'static,
        FnOk: FnOnce(T) -> Msg + Send + 'static,
        FnFail: FnOnce(E, u32) -> Msg + Send + 'static,
    {
        assert!(max_attempts >= 1, "max_attempts must be at least 1");

        Cmd::task(async move {
            let mut delay = initial_delay;
            let mut last_err = None;

            for attempt in 1..=max_attempts {
                match make_future().await {
                    Ok(value) => return on_ok(value),
                    Err(err) => {
                        last_err = Some(err);
                        if attempt < max_attempts {
                            tokio::time::sleep(delay).await;
                            delay *= 2; // exponential backoff
                        }
                    }
                }
            }

            // All attempts failed
            on_fail(last_err.unwrap(), max_attempts)
        })
    }

    /// Retry with custom backoff strategy
    ///
    /// Like `retry`, but allows custom delay calculation.
    ///
    /// # Example
    /// ```ignore
    /// // Linear backoff: 100ms, 200ms, 300ms, ...
    /// Cmd::retry_with_backoff(
    ///     5,
    ///     |attempt| Duration::from_millis(100 * attempt as u64),
    ///     || api::fetch_data(),
    ///     |data| Msg::DataLoaded(data),
    ///     |err, attempts| Msg::FetchFailed(err.to_string(), attempts),
    /// )
    /// ```
    pub fn retry_with_backoff<F, Fut, T, E, B, FnOk, FnFail>(
        max_attempts: u32,
        backoff: B,
        make_future: F,
        on_ok: FnOk,
        on_fail: FnFail,
    ) -> Self
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send,
        T: Send + 'static,
        E: Send + 'static,
        B: Fn(u32) -> std::time::Duration + Send + 'static,
        FnOk: FnOnce(T) -> Msg + Send + 'static,
        FnFail: FnOnce(E, u32) -> Msg + Send + 'static,
    {
        assert!(max_attempts >= 1, "max_attempts must be at least 1");

        Cmd::task(async move {
            let mut last_err = None;

            for attempt in 1..=max_attempts {
                match make_future().await {
                    Ok(value) => return on_ok(value),
                    Err(err) => {
                        last_err = Some(err);
                        if attempt < max_attempts {
                            let delay = backoff(attempt);
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }

            on_fail(last_err.unwrap(), max_attempts)
        })
    }
}
