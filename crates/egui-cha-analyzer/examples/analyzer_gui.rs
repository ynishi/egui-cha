//! egui-cha Analyzer GUI
//!
//! A visual tool for analyzing egui TEA code flow

use egui_cha::prelude::*;
use egui_cha_analyzer::{graph_generator, Analyzer};
use egui_cha_ds::prelude::*;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    egui_cha::run::<AnalyzerApp>(RunConfig::new("egui-cha Analyzer").with_size(1000.0, 700.0))
}

// ============================================================
// App
// ============================================================

struct AnalyzerApp;

#[derive(Default)]
struct Model {
    // Input
    source_code: String,
    file_path: String,
    // Analysis result
    analysis_output: String,
    mermaid_output: String,
    // UI state
    active_tab: Tab,
    // Stats
    tea_flow_count: usize,
    ui_flow_count: usize,
    emission_count: usize,
    handler_count: usize,
}

#[derive(Default, Clone, PartialEq, Debug)]
enum Tab {
    #[default]
    TeaFlows,
    UiFlows,
    Mermaid,
    RawData,
}

#[derive(Clone, Debug)]
enum Msg {
    // Input
    SourceCodeChanged(String),
    FilePathChanged(String),
    // Actions
    Analyze,
    LoadFile,
    LoadExample,
    Clear,
    // Tab
    SetTab(Tab),
}

impl App for AnalyzerApp {
    type Model = Model;
    type Msg = Msg;

    fn init() -> (Model, Cmd<Msg>) {
        let example_code = r#"// Example: TEA Counter
use egui_cha::prelude::*;

enum Msg {
    Increment,
    Decrement,
    Reset,
}

impl App for CounterApp {
    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::Increment => model.count += 1,
            Msg::Decrement => model.count -= 1,
            Msg::Reset => model.count = 0,
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        Button::primary("+").on_click(ctx, Msg::Increment);
        Button::secondary("-").on_click(ctx, Msg::Decrement);
        Button::outline("Reset").on_click(ctx, Msg::Reset);
    }
}"#
        .to_string();

        (
            Model {
                source_code: example_code,
                file_path: "example.rs".to_string(),
                ..Default::default()
            },
            Cmd::msg(Msg::Analyze),
        )
    }

    fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::SourceCodeChanged(code) => {
                model.source_code = code;
            }
            Msg::FilePathChanged(path) => {
                model.file_path = path;
            }
            Msg::Analyze => {
                analyze_code(model);
            }
            Msg::LoadFile => {
                if !model.file_path.is_empty() {
                    match std::fs::read_to_string(&model.file_path) {
                        Ok(content) => {
                            model.source_code = content;
                            return Cmd::msg(Msg::Analyze);
                        }
                        Err(e) => {
                            model.analysis_output = format!("Error loading file: {}", e);
                        }
                    }
                }
            }
            Msg::LoadExample => {
                model.file_path = "examples/counter/src/main.rs".to_string();
                return Cmd::msg(Msg::LoadFile);
            }
            Msg::Clear => {
                model.source_code.clear();
                model.analysis_output.clear();
                model.mermaid_output.clear();
                model.tea_flow_count = 0;
                model.ui_flow_count = 0;
                model.emission_count = 0;
                model.handler_count = 0;
            }
            Msg::SetTab(tab) => {
                model.active_tab = tab;
            }
        }
        Cmd::none()
    }

    fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
        // Use sidebar_layout for clean two-panel design
        ctx.sidebar_layout(
            "analyzer_sidebar",
            450.0,
            // Left panel: Source code input
            |ctx| {
                ctx.ui.heading("Source Code");
                ctx.ui.separator();

                // File path input
                ctx.horizontal(|ctx| {
                    ctx.ui.label("File:");
                    Input::new().placeholder("path/to/file.rs").show_with(
                        ctx,
                        &model.file_path,
                        Msg::FilePathChanged,
                    );
                });

                ctx.horizontal(|ctx| {
                    Button::outline("Load File").on_click(ctx, Msg::LoadFile);
                    Button::secondary("Load Counter").on_click(ctx, Msg::LoadExample);
                });

                ctx.ui.add_space(4.0);

                // Source code text area
                ctx.scroll_area_id("source_code", |ctx| {
                    let mut code = model.source_code.clone();
                    if ctx
                        .ui
                        .add(
                            egui::TextEdit::multiline(&mut code)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(30),
                        )
                        .changed()
                    {
                        ctx.emit(Msg::SourceCodeChanged(code));
                    }
                });

                ctx.ui.add_space(8.0);

                ctx.horizontal(|ctx| {
                    Button::primary("Analyze").on_click(ctx, Msg::Analyze);
                    Button::ghost("Clear").on_click(ctx, Msg::Clear);
                });
            },
            // Right panel: Analysis results
            |ctx| {
                ctx.ui.heading("Analysis Results");
                ctx.ui.separator();

                // Stats
                ctx.horizontal(|ctx| {
                    Badge::info(&format!("TEA: {}", model.tea_flow_count)).show(ctx.ui);
                    Badge::success(&format!("Emissions: {}", model.emission_count)).show(ctx.ui);
                    Badge::warning(&format!("Handlers: {}", model.handler_count)).show(ctx.ui);
                });

                ctx.ui.add_space(4.0);

                // Tabs using DS Tabs component
                Tabs::new(&["TEA Flows", "Mermaid", "Raw Data"]).show_with(
                    ctx,
                    tab_to_index(&model.active_tab),
                    |idx| Msg::SetTab(index_to_tab(idx)),
                );

                ctx.ui.add_space(4.0);

                // Output
                ctx.scroll_area_id("analysis_output", |ctx| {
                    let output = match model.active_tab {
                        Tab::TeaFlows | Tab::UiFlows | Tab::RawData => &model.analysis_output,
                        Tab::Mermaid => &model.mermaid_output,
                    };
                    ctx.ui.add(
                        egui::TextEdit::multiline(&mut output.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                });
            },
        );
    }
}

// ============================================================
// Helpers
// ============================================================

fn tab_to_index(tab: &Tab) -> usize {
    match tab {
        Tab::TeaFlows => 0,
        Tab::Mermaid => 1,
        Tab::UiFlows | Tab::RawData => 2,
    }
}

fn index_to_tab(idx: usize) -> Tab {
    match idx {
        0 => Tab::TeaFlows,
        1 => Tab::Mermaid,
        _ => Tab::RawData,
    }
}

fn analyze_code(model: &mut Model) {
    let analyzer = Analyzer::new();

    match analyzer.analyze_source("input.rs", &model.source_code) {
        Ok(analysis) => {
            // Update stats
            model.tea_flow_count = analysis.tea_flows.len();
            model.ui_flow_count = analysis.flows.len();
            model.emission_count = analysis.msg_emissions.len();
            model.handler_count = analysis.msg_handlers.len();

            // Build output based on active tab
            let mut output = String::new();

            // TEA Flows section
            output.push_str("=== TEA Flows ===\n\n");
            if analysis.tea_flows.is_empty() {
                output.push_str("No TEA flows detected.\n");
            } else {
                for (i, flow) in analysis.tea_flows.iter().enumerate() {
                    let label = flow.emission.label.as_deref().unwrap_or("-");
                    output.push_str(&format!(
                        "Flow {}: {}::{}(\"{}\") -> {} -> {}\n",
                        i + 1,
                        flow.emission.component,
                        flow.emission.variant,
                        label,
                        flow.emission.action,
                        flow.emission.msg
                    ));
                    if let Some(handler) = &flow.handler {
                        output.push_str(&format!(
                            "  -> {} state mutation(s)\n",
                            handler.state_mutations.len()
                        ));
                        for m in &handler.state_mutations {
                            output.push_str(&format!("     {} [{}]\n", m.target, m.mutation_type));
                        }
                    } else {
                        output.push_str("  (no handler found)\n");
                    }
                    output.push('\n');
                }
            }

            // Standard UI Flows section
            output.push_str("\n=== Standard UI Flows ===\n\n");
            if analysis.flows.is_empty() {
                output.push_str("No standard egui flows detected.\n");
            } else {
                for (i, flow) in analysis.flows.iter().enumerate() {
                    let ui_label = flow
                        .ui_element
                        .label
                        .as_deref()
                        .unwrap_or(&flow.ui_element.element_type);
                    output.push_str(&format!(
                        "Flow {}: {} \"{}\" -> .{}()\n",
                        i + 1,
                        flow.ui_element.element_type,
                        ui_label,
                        flow.action.action_type
                    ));
                    for mutation in &flow.state_mutations {
                        output.push_str(&format!(
                            "  -> {} [{}]\n",
                            mutation.target, mutation.mutation_type
                        ));
                    }
                }
            }

            // Raw data section
            output.push_str("\n=== Raw Detections ===\n\n");
            output.push_str(&format!(
                "Msg Emissions: {}\n",
                analysis.msg_emissions.len()
            ));
            for em in &analysis.msg_emissions {
                output.push_str(&format!(
                    "  {}::{}({:?}) -> {} -> {}\n",
                    em.component, em.variant, em.label, em.action, em.msg
                ));
            }
            output.push_str(&format!(
                "\nMsg Handlers: {}\n",
                analysis.msg_handlers.len()
            ));
            for h in &analysis.msg_handlers {
                output.push_str(&format!(
                    "  {} -> {} mutation(s)\n",
                    h.msg_pattern,
                    h.state_mutations.len()
                ));
            }
            output.push_str(&format!("\nUI Elements: {}\n", analysis.ui_elements.len()));
            output.push_str(&format!("Actions: {}\n", analysis.actions.len()));
            output.push_str(&format!(
                "State Mutations: {}\n",
                analysis.state_mutations.len()
            ));

            model.analysis_output = output;

            // Generate Mermaid diagram
            model.mermaid_output = if !analysis.flows.is_empty() {
                graph_generator::generate_flow_mermaid(&analysis)
            } else {
                generate_tea_mermaid(&analysis)
            };
        }
        Err(e) => {
            model.analysis_output = format!("Parse Error:\n{}", e);
            model.mermaid_output.clear();
            model.tea_flow_count = 0;
            model.ui_flow_count = 0;
            model.emission_count = 0;
            model.handler_count = 0;
        }
    }
}

/// Generate Mermaid diagram for TEA flows
fn generate_tea_mermaid(analysis: &egui_cha_analyzer::types::FileAnalysis) -> String {
    if analysis.tea_flows.is_empty() {
        return "flowchart TD\n    %% No TEA flows detected".to_string();
    }

    let mut lines = vec!["flowchart TD".to_string(), "".to_string()];

    for (i, flow) in analysis.tea_flows.iter().enumerate() {
        let flow_id = format!("T{}", i);
        let label = flow.emission.label.as_deref().unwrap_or("-");

        // UI Component node
        let ui_node = format!("{}_UI", flow_id);
        lines.push(format!(
            "    {}[\"{}::{}('{}')\"]",
            ui_node, flow.emission.component, flow.emission.variant, label
        ));
        lines.push(format!("    style {} fill:#e1f5fe", ui_node));

        // Action node
        let act_node = format!("{}_ACT", flow_id);
        lines.push(format!("    {}{{\"{}\"}}", act_node, flow.emission.action));
        lines.push(format!("    style {} fill:#fff9c4", act_node));

        // Msg node
        let msg_node = format!("{}_MSG", flow_id);
        lines.push(format!("    {}((\"{}\" ))", msg_node, flow.emission.msg));
        lines.push(format!("    style {} fill:#ffecb3", msg_node));

        // Connect UI -> Action -> Msg
        lines.push(format!("    {} --> {}", ui_node, act_node));
        lines.push(format!("    {} --> {}", act_node, msg_node));

        // Handler mutations
        if let Some(handler) = &flow.handler {
            for (j, m) in handler.state_mutations.iter().enumerate() {
                let state_node = format!("{}_S{}", flow_id, j);
                lines.push(format!(
                    "    {}([\"{}  [{}]\"])",
                    state_node,
                    m.target.replace('.', " . "),
                    m.mutation_type
                ));
                lines.push(format!("    style {} fill:#c8e6c9", state_node));
                lines.push(format!("    {} --> {}", msg_node, state_node));
            }
        }

        lines.push("".to_string());
    }

    lines.join("\n")
}
