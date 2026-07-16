//! Runtime - Integration with eframe

use crate::{
    error::{FrameworkError, Severity},
    sub::Sub,
    App, Cmd, ViewCtx,
};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::time::Duration;
use tokio::runtime::Runtime as TokioRuntime;
use tokio::task::JoinHandle;

/// Repaint mode for controlling frame rate
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RepaintMode {
    /// Event-driven (default) - only repaint on user input or pending messages.
    /// Most power-efficient, ideal for standard UI applications.
    #[default]
    Reactive,
    /// Fixed FPS - repaint at specified frames per second.
    /// Useful for animations, VJ software, or real-time visualizations.
    FixedFps(u32),
    /// VSync - repaint at monitor refresh rate.
    /// Smooth rendering synced to display, uses more resources.
    VSync,
}

/// Configuration for running the app
pub struct RunConfig {
    /// Window title
    pub title: String,
    /// Initial window size
    pub initial_size: Option<[f32; 2]>,
    /// Enable persistence
    pub persistence: bool,
    /// Repaint mode
    pub repaint_mode: RepaintMode,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            title: "egui-cha App".to_string(),
            initial_size: Some([800.0, 600.0]),
            persistence: false,
            repaint_mode: RepaintMode::default(),
        }
    }
}

impl RunConfig {
    /// Create config with title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set initial window size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.initial_size = Some([width, height]);
        self
    }

    /// Enable persistence
    pub fn with_persistence(mut self) -> Self {
        self.persistence = true;
        self
    }

    /// Set repaint mode
    pub fn with_repaint_mode(mut self, mode: RepaintMode) -> Self {
        self.repaint_mode = mode;
        self
    }
}

/// Run the TEA application
pub fn run<A: App>(config: RunConfig) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(config.initial_size.unwrap_or([800.0, 600.0])),
        ..Default::default()
    };

    let repaint_mode = config.repaint_mode;

    eframe::run_native(
        &config.title,
        options,
        Box::new(move |cc| Ok(Box::new(TeaRuntime::<A>::new(cc, repaint_mode)?))),
    )
}

/// Internal runtime that bridges TEA with eframe
struct TeaRuntime<A: App> {
    model: A::Model,
    pending_msgs: Vec<A::Msg>,
    msg_receiver: mpsc::Receiver<A::Msg>,
    msg_sender: mpsc::Sender<A::Msg>,
    /// Channel for framework errors
    err_receiver: mpsc::Receiver<FrameworkError>,
    err_sender: mpsc::Sender<FrameworkError>,
    tokio_runtime: TokioRuntime,
    /// Active interval subscriptions
    active_intervals: HashMap<&'static str, IntervalHandle>,
    /// Repaint mode
    repaint_mode: RepaintMode,
    /// Egui context handle, used to wake the UI thread when a background
    /// task or interval delivers a message while the app is idle.
    egui_ctx: egui::Context,
}

/// Handle for a running interval
struct IntervalHandle {
    /// Task handle for cancellation via abort()
    handle: JoinHandle<()>,
}

/// Phosphor Icons font (embedded)
const PHOSPHOR_FONT: &[u8] = include_bytes!("../assets/fonts/Phosphor.ttf");

/// Set up icon fonts for egui-cha-ds components.
///
/// Call this when using `eframe::run_native()` directly instead of `egui_cha::run()`.
/// This registers the Phosphor Icons font as `FontFamily::Name("icons")`.
///
/// # Example
///
/// ```rust,ignore
/// eframe::run_native(
///     "My App",
///     options,
///     Box::new(|cc| {
///         egui_cha::setup_icon_fonts(&cc.egui_ctx);
///         Ok(Box::new(MyApp::default()))
///     }),
/// )
/// ```
pub fn setup_icon_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "phosphor".to_owned(),
        egui::FontData::from_static(PHOSPHOR_FONT).into(),
    );

    fonts.families.insert(
        egui::FontFamily::Name("icons".into()),
        vec!["phosphor".to_owned()],
    );

    ctx.set_fonts(fonts);
}

impl<A: App> TeaRuntime<A> {
    fn new(cc: &eframe::CreationContext<'_>, repaint_mode: RepaintMode) -> std::io::Result<Self> {
        // Set up fonts
        setup_icon_fonts(&cc.egui_ctx);

        Self::with_ctx(cc.egui_ctx.clone(), repaint_mode)
    }

    /// Construct the runtime around an existing egui context.
    ///
    /// Split out of [`Self::new`] so tests can drive the runtime without an
    /// `eframe::CreationContext`.
    fn with_ctx(egui_ctx: egui::Context, repaint_mode: RepaintMode) -> std::io::Result<Self> {
        let (model, init_cmd) = A::init();
        let (msg_sender, msg_receiver) = mpsc::channel();
        let (err_sender, err_receiver) = mpsc::channel();

        let tokio_runtime = TokioRuntime::new()?;

        let runtime = Self {
            model,
            pending_msgs: Vec::new(),
            msg_receiver,
            msg_sender,
            err_receiver,
            err_sender,
            tokio_runtime,
            active_intervals: HashMap::new(),
            repaint_mode,
            egui_ctx,
        };

        // Execute initial command
        runtime.execute_cmd(init_cmd);

        Ok(runtime)
    }

    fn execute_cmd(&self, cmd: Cmd<A::Msg>) {
        match cmd {
            Cmd::None => {}
            Cmd::Batch(cmds) => {
                for c in cmds {
                    self.execute_cmd(c);
                }
            }
            Cmd::Task(future) => {
                let msg_sender = self.msg_sender.clone();
                let err_sender = self.err_sender.clone();
                let egui_ctx = self.egui_ctx.clone();

                self.tokio_runtime.spawn(async move {
                    // Catch panics in async tasks
                    let result = tokio::task::spawn(future).await;

                    match result {
                        Ok(msg) => {
                            let _ = msg_sender.send(msg);
                        }
                        Err(join_error) => {
                            // Task panicked or was cancelled
                            let err = if join_error.is_panic() {
                                FrameworkError::command(
                                    Severity::Error,
                                    format!("Task panicked: {}", join_error),
                                )
                            } else {
                                FrameworkError::command(
                                    Severity::Warn,
                                    "Task was cancelled".to_string(),
                                )
                            };
                            let _ = err_sender.send(err);
                        }
                    }

                    // Wake the UI thread so the message is processed even if
                    // the app is idle (Reactive repaint mode).
                    egui_ctx.request_repaint();
                });
            }
            Cmd::Msg(msg) => {
                let _ = self.msg_sender.send(msg);
            }
        }
    }

    fn process_pending_messages(&mut self) {
        // Upper bound on update passes per frame. A `Cmd::Msg` chain is
        // drained within a single frame, but an app whose `update` emits a
        // new message on every pass must not livelock the UI thread.
        const MAX_UPDATE_PASSES: usize = 16;

        for _ in 0..MAX_UPDATE_PASSES {
            // Collect messages from channel
            while let Ok(msg) = self.msg_receiver.try_recv() {
                self.pending_msgs.push(msg);
            }

            if self.pending_msgs.is_empty() {
                return;
            }

            // Process all pending messages
            let msgs = std::mem::take(&mut self.pending_msgs);
            for msg in msgs {
                let cmd = A::update(&mut self.model, msg);
                self.execute_cmd(cmd);
            }
        }

        // Pass budget exhausted: move whatever is left in the channel into
        // pending_msgs so the repaint check schedules the next frame.
        while let Ok(msg) = self.msg_receiver.try_recv() {
            self.pending_msgs.push(msg);
        }
    }

    /// Process framework errors by calling App::on_framework_error
    fn process_framework_errors(&mut self) {
        while let Ok(err) = self.err_receiver.try_recv() {
            let cmd = A::on_framework_error(&mut self.model, err);
            self.execute_cmd(cmd);
        }
    }

    /// Process subscription changes
    fn process_subscriptions(&mut self, sub: Sub<A::Msg>) {
        // Collect current subscription IDs
        let mut current_ids = HashSet::new();
        sub.collect_interval_ids(&mut current_ids);

        // Stop intervals that are no longer in subscriptions
        let to_stop: Vec<_> = self
            .active_intervals
            .keys()
            .filter(|id| !current_ids.contains(*id))
            .copied()
            .collect();

        for id in to_stop {
            self.stop_interval(id);
        }

        // Start new intervals
        for (id, duration, msg) in sub.intervals() {
            if !self.active_intervals.contains_key(id) {
                self.start_interval(id, duration, msg);
            }
        }
    }

    /// Start a new interval
    fn start_interval(&mut self, id: &'static str, duration: Duration, msg: A::Msg) {
        let sender = self.msg_sender.clone();
        let egui_ctx = self.egui_ctx.clone();

        let handle = self.tokio_runtime.spawn(async move {
            let mut interval = tokio::time::interval(duration);
            // Skip first tick (fires immediately)
            interval.tick().await;

            loop {
                interval.tick().await;
                if sender.send(msg.clone()).is_err() {
                    break; // Channel closed, stop interval
                }
                // Wake the UI thread so the tick is processed even if the
                // app is idle (Reactive repaint mode).
                egui_ctx.request_repaint();
            }
        });

        self.active_intervals.insert(id, IntervalHandle { handle });
    }

    /// Stop an interval by ID
    fn stop_interval(&mut self, id: &'static str) {
        if let Some(handle) = self.active_intervals.remove(id) {
            handle.handle.abort();
        }
    }
}

impl<A: App> eframe::App for TeaRuntime<A> {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending messages from commands
        self.process_pending_messages();

        // Process framework errors
        self.process_framework_errors();

        // Process subscriptions (start/stop intervals based on model state)
        let sub = A::subscriptions(&self.model);
        self.process_subscriptions(sub);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Collect messages from view
        let mut view_msgs = Vec::new();

        {
            let mut view_ctx = ViewCtx::new(ui, &mut view_msgs);
            A::view(&self.model, &mut view_ctx);
        }

        // Queue view messages for next frame
        self.pending_msgs.extend(view_msgs);

        // Handle repaint based on mode. Runs after view so pending_msgs
        // reflects newly emitted messages from this frame's view.
        let ctx = ui.ctx();
        match self.repaint_mode {
            RepaintMode::Reactive => {
                // Repaint immediately when messages are already queued.
                // Async tasks and interval ticks wake the UI thread
                // themselves via `Context::request_repaint`, so an idle app
                // with active intervals no longer busy-loops at full
                // framerate.
                if !self.pending_msgs.is_empty() {
                    ctx.request_repaint();
                }
            }
            RepaintMode::FixedFps(fps) => {
                // Request repaint after fixed interval
                let interval = Duration::from_secs_f64(1.0 / fps as f64);
                ctx.request_repaint_after(interval);
            }
            RepaintMode::VSync => {
                // Always request repaint for next vsync
                ctx.request_repaint();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// Poll `cond` until it holds or the timeout expires.
    fn wait_until(timeout: Duration, cond: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if cond() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        cond()
    }

    /// A fresh context starts with a repaint already requested; run a few
    /// empty frames to consume it so tests observe a clean edge.
    fn drained_context() -> egui::Context {
        let ctx = egui::Context::default();
        for _ in 0..8 {
            let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
            if !ctx.has_requested_repaint() {
                break;
            }
        }
        assert!(
            !ctx.has_requested_repaint(),
            "context still has a pending repaint after warm-up frames"
        );
        ctx
    }

    struct ChainApp;

    #[derive(Clone, Debug, PartialEq)]
    enum ChainMsg {
        Start,
        Middle,
        End,
    }

    impl App for ChainApp {
        type Model = Vec<&'static str>;
        type Msg = ChainMsg;

        fn init() -> (Self::Model, Cmd<Self::Msg>) {
            (Vec::new(), Cmd::none())
        }

        fn update(model: &mut Self::Model, msg: Self::Msg) -> Cmd<Self::Msg> {
            match msg {
                ChainMsg::Start => {
                    model.push("start");
                    Cmd::msg(ChainMsg::Middle)
                }
                ChainMsg::Middle => {
                    model.push("middle");
                    Cmd::msg(ChainMsg::End)
                }
                ChainMsg::End => {
                    model.push("end");
                    Cmd::none()
                }
            }
        }

        fn view(_model: &Self::Model, _ctx: &mut ViewCtx<Self::Msg>) {}
    }

    #[test]
    fn cmd_msg_chain_is_drained_within_a_single_frame() {
        let mut runtime =
            TeaRuntime::<ChainApp>::with_ctx(egui::Context::default(), RepaintMode::Reactive)
                .expect("failed to create runtime");

        runtime.pending_msgs.push(ChainMsg::Start);
        runtime.process_pending_messages();

        assert_eq!(runtime.model, vec!["start", "middle", "end"]);
        assert!(runtime.pending_msgs.is_empty());
    }

    struct SelfFeedingApp;

    impl App for SelfFeedingApp {
        type Model = u32;
        type Msg = ();

        fn init() -> (Self::Model, Cmd<Self::Msg>) {
            (0, Cmd::none())
        }

        fn update(model: &mut Self::Model, _msg: Self::Msg) -> Cmd<Self::Msg> {
            *model += 1;
            Cmd::msg(())
        }

        fn view(_model: &Self::Model, _ctx: &mut ViewCtx<Self::Msg>) {}
    }

    #[test]
    fn self_feeding_update_does_not_livelock_a_frame() {
        let mut runtime =
            TeaRuntime::<SelfFeedingApp>::with_ctx(egui::Context::default(), RepaintMode::Reactive)
                .expect("failed to create runtime");

        runtime.pending_msgs.push(());
        runtime.process_pending_messages();

        // A bounded number of passes ran, and the leftover message is queued
        // for the next frame instead of spinning forever.
        assert!(runtime.model >= 1);
        assert!(!runtime.pending_msgs.is_empty());
    }

    struct NoopApp;

    impl App for NoopApp {
        type Model = ();
        type Msg = ();

        fn init() -> (Self::Model, Cmd<Self::Msg>) {
            ((), Cmd::none())
        }

        fn update(_model: &mut Self::Model, _msg: Self::Msg) -> Cmd<Self::Msg> {
            Cmd::none()
        }

        fn view(_model: &Self::Model, _ctx: &mut ViewCtx<Self::Msg>) {}
    }

    #[test]
    fn task_completion_requests_repaint() {
        let ctx = drained_context();
        let runtime = TeaRuntime::<NoopApp>::with_ctx(ctx.clone(), RepaintMode::Reactive)
            .expect("failed to create runtime");

        runtime.execute_cmd(Cmd::task(async {}));

        // The task sends its message and then wakes the UI thread.
        assert!(
            wait_until(Duration::from_secs(5), || ctx.has_requested_repaint()),
            "task completion did not request a repaint"
        );
        assert!(runtime.msg_receiver.try_recv().is_ok());
    }

    #[test]
    fn interval_tick_requests_repaint() {
        let ctx = drained_context();
        let mut runtime = TeaRuntime::<NoopApp>::with_ctx(ctx.clone(), RepaintMode::Reactive)
            .expect("failed to create runtime");

        runtime.start_interval("test", Duration::from_millis(10), ());

        // The first tick is skipped; the next one sends and wakes the UI.
        assert!(
            wait_until(Duration::from_secs(5), || ctx.has_requested_repaint()),
            "interval tick did not request a repaint"
        );
        assert!(runtime.msg_receiver.try_recv().is_ok());
        runtime.stop_interval("test");
    }
}
