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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepaintMode {
    /// Event-driven (default) - only repaint on user input or pending messages.
    /// Most power-efficient, ideal for standard UI applications.
    Reactive,
    /// Fixed FPS - repaint at specified frames per second.
    /// Useful for animations, VJ software, or real-time visualizations.
    FixedFps(u32),
    /// VSync - repaint at monitor refresh rate.
    /// Smooth rendering synced to display, uses more resources.
    VSync,
}

impl Default for RepaintMode {
    fn default() -> Self {
        Self::Reactive
    }
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
        Box::new(move |cc| Ok(Box::new(TeaRuntime::<A>::new(cc, repaint_mode)))),
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
}

/// Handle for a running interval
struct IntervalHandle {
    /// Task handle for cancellation via abort()
    handle: JoinHandle<()>,
}

/// Phosphor Icons font (embedded)
const PHOSPHOR_FONT: &[u8] = include_bytes!("../../../assets/fonts/Phosphor.ttf");

impl<A: App> TeaRuntime<A> {
    fn new(cc: &eframe::CreationContext<'_>, repaint_mode: RepaintMode) -> Self {
        // Set up fonts
        Self::setup_fonts(&cc.egui_ctx);

        let (model, init_cmd) = A::init();
        let (msg_sender, msg_receiver) = mpsc::channel();
        let (err_sender, err_receiver) = mpsc::channel();

        let tokio_runtime = TokioRuntime::new().expect("Failed to create tokio runtime");

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
        };

        // Execute initial command
        runtime.execute_cmd(init_cmd);

        runtime
    }

    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Add Phosphor Icons font
        fonts.font_data.insert(
            "phosphor".to_owned(),
            egui::FontData::from_static(PHOSPHOR_FONT).into(),
        );

        // Register as a named font family
        fonts.families.insert(
            egui::FontFamily::Name("icons".into()),
            vec!["phosphor".to_owned()],
        );

        ctx.set_fonts(fonts);
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
                });
            }
            Cmd::Msg(msg) => {
                let _ = self.msg_sender.send(msg);
            }
        }
    }

    fn process_pending_messages(&mut self) {
        // Collect messages from channel
        while let Ok(msg) = self.msg_receiver.try_recv() {
            self.pending_msgs.push(msg);
        }

        // Process all pending messages
        let msgs = std::mem::take(&mut self.pending_msgs);
        for msg in msgs {
            let cmd = A::update(&mut self.model, msg);
            self.execute_cmd(cmd);
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

        let handle = self.tokio_runtime.spawn(async move {
            let mut interval = tokio::time::interval(duration);
            // Skip first tick (fires immediately)
            interval.tick().await;

            loop {
                interval.tick().await;
                if sender.send(msg.clone()).is_err() {
                    break; // Channel closed, stop interval
                }
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending messages from commands
        self.process_pending_messages();

        // Process framework errors
        self.process_framework_errors();

        // Process subscriptions (start/stop intervals based on model state)
        let sub = A::subscriptions(&self.model);
        self.process_subscriptions(sub);

        // Collect messages from view
        let mut view_msgs = Vec::new();

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut view_ctx = ViewCtx::new(ui, &mut view_msgs);
            A::view(&self.model, &mut view_ctx);
        });

        // Queue view messages for next frame
        self.pending_msgs.extend(view_msgs);

        // Handle repaint based on mode
        match self.repaint_mode {
            RepaintMode::Reactive => {
                // Only repaint if there are pending messages or active intervals
                if !self.pending_msgs.is_empty() || !self.active_intervals.is_empty() {
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
