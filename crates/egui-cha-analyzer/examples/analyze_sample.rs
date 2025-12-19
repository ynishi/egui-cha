//! Example: Analyze sample egui code and generate Mermaid graph

use egui_cha_analyzer::{graph_generator, Analyzer};

fn main() {
    // Sample egui code to analyze
    let sample_code = r#"
struct AppState {
    counter: i32,
    name: String,
    enabled: bool,
    items: Vec<String>,
}

fn show_ui(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Counter App");

    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            state.counter -= 1;
        }

        ui.label(format!("{}", state.counter));

        if ui.button("+").clicked() {
            state.counter += 1;
        }
    });

    ui.separator();

    ui.text_edit_singleline(&mut state.name);

    if ui.checkbox(&mut state.enabled, "Enable feature").changed() {
        println!("Feature toggled");
    }

    if ui.button("Add Item").clicked() {
        state.items.push("New Item".to_string());
    }

    if ui.button("Clear").clicked() {
        state.items.clear();
    }

    if ui.button("Reset All").clicked() {
        state.counter = 0;
        state.name = String::new();
        state.enabled = false;
    }
}
"#;

    let analyzer = Analyzer::new();

    match analyzer.analyze_source("sample.rs", sample_code) {
        Ok(analysis) => {
            println!("=== Analysis Result ===\n");

            println!("UI Elements ({}):", analysis.ui_elements.len());
            for ui in &analysis.ui_elements {
                let label = ui.label.as_deref().unwrap_or("-");
                println!("  - {} \"{}\" in {}()", ui.element_type, label, ui.context);
            }

            println!("\nActions ({}):", analysis.actions.len());
            for action in &analysis.actions {
                println!("  - {}.{}() in {}()", action.source, action.action_type, action.context);
            }

            println!("\nState Mutations ({}):", analysis.state_mutations.len());
            for mutation in &analysis.state_mutations {
                println!("  - {} [{}] in {}()", mutation.target, mutation.mutation_type, mutation.context);
            }

            println!("\n=== UI Flows (Scope-Aware) ({}) ===\n", analysis.flows.len());
            for (i, flow) in analysis.flows.iter().enumerate() {
                let ui_label = flow.ui_element.label.as_deref().unwrap_or(&flow.ui_element.element_type);
                println!("Flow {}: {} \"{}\" -> .{}()", i + 1, flow.ui_element.element_type, ui_label, flow.action.action_type);
                for mutation in &flow.state_mutations {
                    println!("    -> {} [{}]", mutation.target, mutation.mutation_type);
                }
            }

            println!("\n=== Mermaid Graph (Precise Flows) ===\n");
            let mermaid = graph_generator::generate_flow_mermaid(&analysis);
            println!("{}", mermaid);

            println!("\n=== Copy above to https://mermaid.live to visualize ===");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
