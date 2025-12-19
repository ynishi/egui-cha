//! Example: Analyze a real egui file

use egui_cha_analyzer::{graph_generator, Analyzer};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        args[1].clone()
    } else {
        // Default: analyze counter example
        "examples/counter/src/main.rs".to_string()
    };

    println!("Analyzing: {}\n", file_path);

    let analyzer = Analyzer::new();

    match analyzer.analyze_file(&file_path) {
        Ok(analysis) => {
            println!("=== UI Flows ({}) ===\n", analysis.flows.len());

            if analysis.flows.is_empty() {
                println!("No flows detected.");
                println!("\nNote: This analyzer detects standard egui patterns like:");
                println!("  - if ui.button(\"x\").clicked() {{ state.y = z }}");
                println!("  - let r = ui.button(\"x\"); if r.clicked() {{ ... }}");
                println!("\nDS component patterns like Button::primary().on_click() are not yet supported.");
            } else {
                for (i, flow) in analysis.flows.iter().enumerate() {
                    let ui_label = flow.ui_element.label.as_deref().unwrap_or(&flow.ui_element.element_type);
                    println!(
                        "Flow {}: {} \"{}\" -> .{}()",
                        i + 1,
                        flow.ui_element.element_type,
                        ui_label,
                        flow.action.action_type
                    );
                    for mutation in &flow.state_mutations {
                        println!("    -> {} [{}]", mutation.target, mutation.mutation_type);
                    }
                }

                println!("\n=== Mermaid Graph ===\n");
                let mermaid = graph_generator::generate_flow_mermaid(&analysis);
                println!("{}", mermaid);
            }

            // Also show raw detections
            println!("\n=== Raw Detections ===");
            println!("UI Elements: {}", analysis.ui_elements.len());
            println!("Actions: {}", analysis.actions.len());
            println!("State Mutations: {}", analysis.state_mutations.len());

            // TEA patterns
            println!("\n=== TEA Patterns ===");
            println!("Msg Emissions: {}", analysis.msg_emissions.len());
            for em in &analysis.msg_emissions {
                println!("  {}::{}({:?}) -> {} -> {}",
                    em.component, em.variant, em.label, em.action, em.msg);
            }

            println!("\nMsg Handlers: {}", analysis.msg_handlers.len());
            for h in &analysis.msg_handlers {
                println!("  {} -> {} mutations", h.msg_pattern, h.state_mutations.len());
                for m in &h.state_mutations {
                    println!("    -> {} [{}]", m.target, m.mutation_type);
                }
            }

            println!("\n=== TEA Flows (Complete) ===");
            for (i, flow) in analysis.tea_flows.iter().enumerate() {
                let label = flow.emission.label.as_deref().unwrap_or("-");
                print!("Flow {}: {}::{}(\"{}\") -> {} -> {}",
                    i + 1,
                    flow.emission.component,
                    flow.emission.variant,
                    label,
                    flow.emission.action,
                    flow.emission.msg
                );
                if let Some(handler) = &flow.handler {
                    println!(" -> {} mutations", handler.state_mutations.len());
                    for m in &handler.state_mutations {
                        println!("    -> {} [{}]", m.target, m.mutation_type);
                    }
                } else {
                    println!(" (no handler found)");
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
