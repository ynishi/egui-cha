//! Storybook - Component Gallery for egui-cha-ds
//!
//! Showcases all available components with interactive demos.

use egui::Color32;
use egui_cha::prelude::*;
use egui_cha_ds::prelude::*;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    egui_cha::run::<StorybookApp>(
        RunConfig::new("egui-cha-ds Storybook")
            .with_size(1400.0, 900.0)
            .with_repaint_mode(RepaintMode::FixedFps(60)),
    )
}

struct StorybookApp;

#[derive(Clone, Debug)]
enum Msg {
    SelectSection(Section),
    // WorkspaceCanvas
    WorkspaceEvent(WorkspaceEvent),
    ToggleLock,
    SetLayoutMode(bool), // true = Tile, false = Free
    // LayerStack
    LayerEvent(LayerEvent),
    // EffectRack
    EffectEvent(RackEvent),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Section {
    Workspace,
    LayerStack,
    EffectRack,
    ClipGrid,
    Timeline,
}

impl Section {
    fn all() -> &'static [Section] {
        &[
            Section::Workspace,
            Section::LayerStack,
            Section::EffectRack,
            Section::ClipGrid,
            Section::Timeline,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Section::Workspace => "WorkspaceCanvas",
            Section::LayerStack => "LayerStack",
            Section::EffectRack => "EffectRack",
            Section::ClipGrid => "ClipGrid",
            Section::Timeline => "Timeline",
        }
    }
}

struct Model {
    current_section: Section,
    // WorkspaceCanvas
    workspace_panes: Vec<WorkspacePane>,
    workspace_locked: bool,
    workspace_tile_mode: bool,
    // LayerStack
    layers: Vec<Layer>,
    // EffectRack
    effects: Vec<Effect>,
    // ClipGrid
    clips: Vec<ClipCell>,
    current_clip: Option<usize>,
    // Timeline
    timeline_position: f64,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            current_section: Section::Workspace,
            workspace_panes: vec![
                WorkspacePane::new("preview", "Preview")
                    .with_position(20.0, 20.0)
                    .with_size(300.0, 200.0)
                    .with_order(0),
                WorkspacePane::new("effects", "Effects")
                    .with_position(340.0, 20.0)
                    .with_size(250.0, 200.0)
                    .with_order(1),
                WorkspacePane::new("layers", "Layers")
                    .with_position(20.0, 240.0)
                    .with_size(300.0, 180.0)
                    .with_order(2),
                WorkspacePane::new("timeline", "Timeline")
                    .with_position(340.0, 240.0)
                    .with_size(400.0, 100.0)
                    .with_order(3),
            ],
            workspace_locked: false,
            workspace_tile_mode: true,
            layers: vec![
                Layer::new("Main Output").with_opacity(1.0).with_visible(true),
                Layer::new("Overlay").with_opacity(0.8).with_visible(true),
                Layer::new("Background").with_opacity(0.5).with_visible(false),
            ],
            effects: vec![
                Effect::new("Blend", EffectCategory::Utility)
                    .enabled(true)
                    .with_param(EffectParam::new("opacity", 0.5)),
                Effect::new("Glitch", EffectCategory::Distortion)
                    .enabled(false)
                    .with_param(EffectParam::new("intensity", 0.3)),
                Effect::new("Blur", EffectCategory::Time)
                    .enabled(true)
                    .with_param(EffectParam::new("amount", 0.2)),
            ],
            clips: vec![
                ClipCell::new("Intro").with_color(Color32::from_rgb(100, 150, 255)),
                ClipCell::new("Build").with_color(Color32::from_rgb(150, 200, 100)),
                ClipCell::new("Drop").with_color(Color32::from_rgb(255, 100, 100)).with_state(ClipState::Playing),
                ClipCell::new("Break").with_color(Color32::from_rgb(255, 180, 100)),
            ],
            current_clip: Some(2),
            timeline_position: 0.3,
        }
    }
}

impl App for StorybookApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        (Model::default(), Cmd::none())
    }

    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::SelectSection(section) => {
                model.current_section = section;
            }
            Msg::WorkspaceEvent(event) => {
                match event {
                    WorkspaceEvent::PaneMoved { id, position } => {
                        if let Some(pane) = model.workspace_panes.iter_mut().find(|p| p.id == id) {
                            pane.position = position;
                        }
                    }
                    WorkspaceEvent::PaneResized { id, size } => {
                        if let Some(pane) = model.workspace_panes.iter_mut().find(|p| p.id == id) {
                            pane.size = size;
                        }
                    }
                    WorkspaceEvent::PaneClosed(id) => {
                        model.workspace_panes.retain(|p| p.id != id);
                    }
                    WorkspaceEvent::PaneMinimized { id, minimized } => {
                        if let Some(pane) = model.workspace_panes.iter_mut().find(|p| p.id == id) {
                            pane.minimized = minimized;
                        }
                    }
                    WorkspaceEvent::PaneReordered { from, to } => {
                        // Swap the orders of the two panes
                        // First find indices of panes with these orders
                        let idx_from = model.workspace_panes.iter().position(|p| p.order == from);
                        let idx_to = model.workspace_panes.iter().position(|p| p.order == to);

                        if let (Some(i), Some(j)) = (idx_from, idx_to) {
                            // Swap their orders
                            model.workspace_panes[i].order = to;
                            model.workspace_panes[j].order = from;
                        }
                    }
                    WorkspaceEvent::WeightsChanged(weights) => {
                        // Update pane weights
                        for (id, weight) in weights {
                            if let Some(pane) = model.workspace_panes.iter_mut().find(|p| p.id == id) {
                                pane.weight = weight;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Msg::ToggleLock => {
                model.workspace_locked = !model.workspace_locked;
            }
            Msg::SetLayoutMode(tile) => {
                model.workspace_tile_mode = tile;
            }
            Msg::LayerEvent(event) => {
                match event {
                    LayerEvent::ToggleVisible(idx) => {
                        if let Some(l) = model.layers.get_mut(idx) {
                            l.visible = !l.visible;
                        }
                    }
                    LayerEvent::SetOpacity(idx, opacity) => {
                        if let Some(l) = model.layers.get_mut(idx) {
                            l.opacity = opacity;
                        }
                    }
                    LayerEvent::Reorder { from, to } => {
                        if from < model.layers.len() {
                            let layer = model.layers.remove(from);
                            let insert_idx = if to > from { to - 1 } else { to };
                            model.layers.insert(insert_idx.min(model.layers.len()), layer);
                        }
                    }
                    _ => {}
                }
            }
            Msg::EffectEvent(event) => {
                match event {
                    RackEvent::Toggle(idx) => {
                        if let Some(e) = model.effects.get_mut(idx) {
                            e.enabled = !e.enabled;
                        }
                    }
                    RackEvent::Remove(idx) => {
                        if idx < model.effects.len() {
                            model.effects.remove(idx);
                        }
                    }
                    RackEvent::Reorder(from, to) => {
                        if from < model.effects.len() {
                            let effect = model.effects.remove(from);
                            let insert_idx = if to > from { to - 1 } else { to };
                            model.effects.insert(insert_idx.min(model.effects.len()), effect);
                        }
                    }
                    _ => {}
                }
            }
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        let egui_ctx = ctx.ui.ctx().clone();
        Theme::dark().apply(&egui_ctx);

        // Collect sidebar clicks
        let mut sidebar_click: Option<Section> = None;

        // Sidebar
        egui::SidePanel::left("sidebar")
            .min_width(180.0)
            .show(&egui_ctx, |ui| {
                ui.heading("Storybook");
                ui.separator();
                ui.add_space(8.0);

                for section in Section::all() {
                    let selected = model.current_section == *section;
                    if ui.selectable_label(selected, section.label()).clicked() {
                        sidebar_click = Some(*section);
                    }
                }
            });

        // Handle sidebar click
        if let Some(section) = sidebar_click {
            ctx.emit(Msg::SelectSection(section));
        }

        // Main content
        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            match model.current_section {
                Section::Workspace => render_workspace_demo(model, ui, ctx),
                Section::LayerStack => render_layer_stack_demo(model, ui, ctx),
                Section::EffectRack => render_effect_rack_demo(model, ui, ctx),
                Section::ClipGrid => render_clip_grid_demo(model, ui),
                Section::Timeline => render_timeline_demo(model, ui),
            }
        });
    }
}

fn render_workspace_demo(model: &Model, ui: &mut egui::Ui, ctx: &mut ViewCtx<Msg>) {
    ui.heading("WorkspaceCanvas");
    ui.label("Flexible pane layout with Tile/Free modes and Lock support");
    ui.separator();

    // Controls
    ui.horizontal(|ui| {
        ui.label("Mode:");
        if ui.selectable_label(model.workspace_tile_mode, "Tile").clicked() {
            ctx.emit(Msg::SetLayoutMode(true));
        }
        if ui.selectable_label(!model.workspace_tile_mode, "Free").clicked() {
            ctx.emit(Msg::SetLayoutMode(false));
        }

        ui.add_space(16.0);

        let lock_label = if model.workspace_locked { "Unlock" } else { "Lock" };
        if ui.button(lock_label).clicked() {
            ctx.emit(Msg::ToggleLock);
        }

        ui.add_space(16.0);
        ui.label(format!("Panes: {}", model.workspace_panes.len()));
    });

    ui.add_space(8.0);

    // Note: We need mutable access to panes, so we'll collect events
    // and handle them via message passing
    let layout_mode = if model.workspace_tile_mode {
        LayoutMode::Tile { columns: None }
    } else {
        LayoutMode::Free
    };

    // Clone panes for the canvas (WorkspaceCanvas needs &mut)
    let mut panes = model.workspace_panes.clone();

    let events = WorkspaceCanvas::new(&mut panes)
        .layout(layout_mode)
        .locked(model.workspace_locked)
        .snap_threshold(8.0)
        .gap(8.0)
        .show(ui, |ui, pane| {
            // Render pane content based on id
            match pane.id.as_str() {
                "preview" => {
                    ui.colored_label(Color32::from_rgb(100, 200, 255), "Preview Area");
                    ui.label("Video output preview");
                }
                "effects" => {
                    ui.colored_label(Color32::from_rgb(255, 150, 100), "Effects");
                    ui.label("Effect chain");
                }
                "layers" => {
                    ui.colored_label(Color32::from_rgb(100, 255, 150), "Layers");
                    ui.label("Layer stack");
                }
                "timeline" => {
                    ui.colored_label(Color32::from_rgb(255, 255, 100), "Timeline");
                    ui.label("Time-based controls");
                }
                _ => {
                    ui.label(&pane.title);
                }
            }
        });

    for event in events {
        ctx.emit(Msg::WorkspaceEvent(event));
    }
}

fn render_layer_stack_demo(model: &Model, ui: &mut egui::Ui, ctx: &mut ViewCtx<Msg>) {
    ui.heading("LayerStack");
    ui.label("Drag-to-reorder layers with visibility and opacity controls");
    ui.separator();

    ui.add_space(8.0);
    ui.label("Drag layers to reorder. Locked layers cannot be dragged.");
    ui.add_space(8.0);

    if let Some(event) = LayerStack::new(&model.layers)
        .selected(Some(0))
        .show(ui)
    {
        ctx.emit(Msg::LayerEvent(event));
    }
}

fn render_effect_rack_demo(model: &Model, ui: &mut egui::Ui, ctx: &mut ViewCtx<Msg>) {
    ui.heading("EffectRack");
    ui.label("Drag-to-reorder effects with enable/disable toggles");
    ui.separator();

    ui.add_space(8.0);
    ui.label("Drag effects to reorder. Right-click to remove.");
    ui.add_space(8.0);

    if let Some(event) = EffectRack::new(&model.effects)
        .orientation(RackOrientation::Vertical)
        .effect_size(350.0, 70.0)
        .draggable(true)
        .show(ui)
    {
        ctx.emit(Msg::EffectEvent(event));
    }
}

fn render_clip_grid_demo(model: &Model, ui: &mut egui::Ui) {
    ui.heading("ClipGrid");
    ui.label("Ableton Live-style clip launcher");
    ui.separator();

    ui.add_space(8.0);

    ClipGrid::new(&model.clips, 4)
        .current(model.current_clip)
        .cell_size(100.0, 60.0)
        .show(ui);
}

fn render_timeline_demo(model: &Model, ui: &mut egui::Ui) {
    ui.heading("Timeline");
    ui.label("Seekable timeline with markers and regions");
    ui.separator();

    ui.add_space(8.0);

    let markers = vec![
        TimelineMarker::new(0.0, "Start"),
        TimelineMarker::new(0.25, "A").with_color(Color32::from_rgb(100, 200, 100)),
        TimelineMarker::new(0.5, "B").with_color(Color32::from_rgb(200, 100, 200)),
        TimelineMarker::new(0.75, "C").with_color(Color32::from_rgb(200, 200, 100)),
    ];

    Timeline::new(120.0)
        .position(model.timeline_position)
        .markers(&markers)
        .height(40.0)
        .show(ui);
}
