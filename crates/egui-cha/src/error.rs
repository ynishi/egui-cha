//! Error handling types for egui-cha framework
//!
//! Provides unified error handling with:
//! - [`Severity`] - Unified severity levels compatible with tracing
//! - [`FrameworkError`] - Errors from framework internals
//! - [`ErrorSource`] - Categorization of error origins

/// Unified severity level for both tracing and ErrorChannel
///
/// Ordered from least to most severe for comparison operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Severity {
    /// Development debugging (shown only in debug mode)
    Debug,
    /// Informational message
    #[default]
    Info,
    /// Warning (operation continues)
    Warn,
    /// Error (recoverable)
    Error,
    /// Critical error (may require restart)
    Critical,
}

impl Severity {
    /// Convert to tracing::Level
    pub fn to_tracing_level(self) -> tracing::Level {
        match self {
            Severity::Debug => tracing::Level::DEBUG,
            Severity::Info => tracing::Level::INFO,
            Severity::Warn => tracing::Level::WARN,
            Severity::Error | Severity::Critical => tracing::Level::ERROR,
        }
    }

    /// Check if this severity should be shown in production
    pub fn is_production_visible(self) -> bool {
        self >= Severity::Info
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Debug => write!(f, "DEBUG"),
            Severity::Info => write!(f, "INFO"),
            Severity::Warn => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Where the error originated within the framework
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorSource {
    /// Command execution (Cmd::Task failed, panic, etc.)
    Command,
    /// Runtime internals (tokio, channel, etc.)
    Runtime,
    /// Subscription handling
    Subscription,
    /// View rendering
    View,
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSource::Command => write!(f, "Command"),
            ErrorSource::Runtime => write!(f, "Runtime"),
            ErrorSource::Subscription => write!(f, "Subscription"),
            ErrorSource::View => write!(f, "View"),
        }
    }
}

/// Error from egui-cha framework internals
///
/// This type represents errors that occur within the framework itself,
/// such as task panics, runtime failures, or subscription errors.
///
/// # Example
/// ```ignore
/// fn on_framework_error(model: &mut Model, err: FrameworkError) -> Cmd<Msg> {
///     // Add to your error console
///     model.errors.push_with_level(&err.message, err.severity.into());
///
///     // Or handle specific sources differently
///     match err.source {
///         ErrorSource::Command => { /* retry logic */ }
///         _ => { /* log and continue */ }
///     }
///     Cmd::none()
/// }
/// ```
#[derive(Clone, Debug)]
pub struct FrameworkError {
    /// Error severity
    pub severity: Severity,
    /// Error source/category
    pub source: ErrorSource,
    /// Error message
    pub message: String,
    /// Optional context (additional details)
    pub context: Option<String>,
}

impl FrameworkError {
    /// Create a new framework error
    pub fn new(severity: Severity, source: ErrorSource, message: impl Into<String>) -> Self {
        Self {
            severity,
            source,
            message: message.into(),
            context: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }

    /// Create a command error
    pub fn command(severity: Severity, message: impl Into<String>) -> Self {
        Self::new(severity, ErrorSource::Command, message)
    }

    /// Create a runtime error
    pub fn runtime(severity: Severity, message: impl Into<String>) -> Self {
        Self::new(severity, ErrorSource::Runtime, message)
    }

    /// Create a subscription error
    pub fn subscription(severity: Severity, message: impl Into<String>) -> Self {
        Self::new(severity, ErrorSource::Subscription, message)
    }

    /// Create a view error
    pub fn view(severity: Severity, message: impl Into<String>) -> Self {
        Self::new(severity, ErrorSource::View, message)
    }

    /// Log this error to tracing
    pub fn log(&self) {
        let msg = self.format_message();

        match self.severity {
            Severity::Debug => tracing::debug!("{}", msg),
            Severity::Info => tracing::info!("{}", msg),
            Severity::Warn => tracing::warn!("{}", msg),
            Severity::Error => tracing::error!("{}", msg),
            Severity::Critical => tracing::error!("[CRITICAL] {}", msg),
        }
    }

    /// Format the error message with source and context
    pub fn format_message(&self) -> String {
        match &self.context {
            Some(ctx) => format!("[{}] {} ({})", self.source, self.message, ctx),
            None => format!("[{}] {}", self.source, self.message),
        }
    }
}

impl std::fmt::Display for FrameworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_message())
    }
}

impl std::error::Error for FrameworkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Debug < Severity::Info);
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn severity_production_visibility() {
        assert!(!Severity::Debug.is_production_visible());
        assert!(Severity::Info.is_production_visible());
        assert!(Severity::Warn.is_production_visible());
        assert!(Severity::Error.is_production_visible());
        assert!(Severity::Critical.is_production_visible());
    }

    #[test]
    fn framework_error_formatting() {
        let err = FrameworkError::command(Severity::Error, "Task failed");
        assert_eq!(err.format_message(), "[Command] Task failed");

        let err_with_ctx = err.with_context("user_id=123");
        assert_eq!(
            err_with_ctx.format_message(),
            "[Command] Task failed (user_id=123)"
        );
    }
}
