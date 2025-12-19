//! Runtime - Integration with eframe

use crate::{App, Cmd, ViewCtx};
use std::sync::mpsc;
use tokio::runtime::Runtime as TokioRuntime;

/// Configuration for running the app
pub struct RunConfig {
    /// Window title
    pub title: String,
    /// Initial window size
    pub initial_size: Option<[f32; 2]>,
    /// Enable persistence
    pub persistence: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            title: "egui-cha App".to_string(),
            initial_size: Some([800.0, 600.0]),
            persistence: false,
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
}

/// Run the TEA application
pub fn run<A: App>(config: RunConfig) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(config.initial_size.unwrap_or([800.0, 600.0])),
        ..Default::default()
    };

    eframe::run_native(
        &config.title,
        options,
        Box::new(|cc| Ok(Box::new(TeaRuntime::<A>::new(cc)))),
    )
}

/// Internal runtime that bridges TEA with eframe
struct TeaRuntime<A: App> {
    model: A::Model,
    pending_msgs: Vec<A::Msg>,
    msg_receiver: mpsc::Receiver<A::Msg>,
    msg_sender: mpsc::Sender<A::Msg>,
    tokio_runtime: TokioRuntime,
}

/// Phosphor Icons font (embedded)
const PHOSPHOR_FONT: &[u8] = include_bytes!("../../../assets/fonts/Phosphor.ttf");

impl<A: App> TeaRuntime<A> {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up fonts
        Self::setup_fonts(&cc.egui_ctx);

        let (model, init_cmd) = A::init();
        let (msg_sender, msg_receiver) = mpsc::channel();

        let tokio_runtime = TokioRuntime::new().expect("Failed to create tokio runtime");

        let runtime = Self {
            model,
            pending_msgs: Vec::new(),
            msg_receiver,
            msg_sender,
            tokio_runtime,
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
                let sender = self.msg_sender.clone();
                self.tokio_runtime.spawn(async move {
                    let msg = future.await;
                    let _ = sender.send(msg);
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
}

impl<A: App> eframe::App for TeaRuntime<A> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending messages from commands
        self.process_pending_messages();

        // Execute subscriptions
        let sub_cmd = A::subscriptions(&self.model);
        self.execute_cmd(sub_cmd);

        // Collect messages from view
        let mut view_msgs = Vec::new();

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut view_ctx = ViewCtx::new(ui, &mut view_msgs);
            A::view(&self.model, &mut view_ctx);
        });

        // Queue view messages for next frame
        self.pending_msgs.extend(view_msgs);

        // Request repaint if there are pending messages or async tasks
        if !self.pending_msgs.is_empty() {
            ctx.request_repaint();
        }
    }
}
