//! Generate Mermaid flowcharts from analysis results

use crate::types::{AnalysisResult, FileAnalysis};

/// Generate a precise Mermaid flowchart from flows (scope-aware)
pub fn generate_flow_mermaid(analysis: &FileAnalysis) -> String {
    if analysis.flows.is_empty() {
        return "flowchart TD\n    %% No flows detected".to_string();
    }

    let mut lines = vec![
        "flowchart TD".to_string(),
        "".to_string(),
    ];

    // Generate nodes and connections for each flow
    for (i, flow) in analysis.flows.iter().enumerate() {
        let flow_id = format!("F{}", i);

        // UI element node
        let ui_label = flow.ui_element.label.as_deref().unwrap_or(&flow.ui_element.element_type);
        let ui_icon = ui_icon(&flow.ui_element.element_type);
        let ui_node = format!("{}_UI", flow_id);
        lines.push(format!(
            "    {}[\"{} {}\"]",
            ui_node, ui_icon, escape_mermaid(ui_label)
        ));
        lines.push(format!("    style {} fill:#e1f5fe", ui_node));

        // Action node
        let act_node = format!("{}_ACT", flow_id);
        lines.push(format!(
            "    {}{{\"{}\"}}",
            act_node,
            escape_mermaid(&flow.action.action_type)
        ));
        lines.push(format!("    style {} fill:#fff9c4", act_node));

        // Connect UI -> Action
        lines.push(format!("    {} --> {}", ui_node, act_node));

        // State mutation nodes
        for mutation in &flow.state_mutations {
            let state_node = format!("{}_{}", flow_id, sanitize_id(&mutation.target));
            let icon = mutation_icon(&mutation.mutation_type);
            lines.push(format!(
                "    {}([\"{} {}\"])",
                state_node, icon, escape_mermaid(&mutation.target)
            ));
            lines.push(format!("    style {} fill:#c8e6c9", state_node));

            // Connect Action -> State
            lines.push(format!("    {} --> {}", act_node, state_node));
        }

        lines.push("".to_string());
    }

    lines.join("\n")
}

/// Sanitize a string to be used as a Mermaid node ID
fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Generate a Mermaid flowchart from a file analysis
pub fn generate_mermaid(analysis: &FileAnalysis) -> String {
    let mut lines = vec![
        "flowchart TD".to_string(),
        "".to_string(),
        "    %% UI Elements".to_string(),
    ];

    // Generate UI element nodes
    for (i, ui) in analysis.ui_elements.iter().enumerate() {
        let label = ui.label.as_deref().unwrap_or(&ui.element_type);
        let node_id = format!("UI{}", i);
        let icon = ui_icon(&ui.element_type);
        lines.push(format!(
            "    {}[\"{} {}\"]",
            node_id, icon, escape_mermaid(label)
        ));
        lines.push(format!("    style {} fill:#e1f5fe", node_id));
    }

    lines.push("".to_string());
    lines.push("    %% Actions".to_string());

    // Generate action nodes
    for (i, action) in analysis.actions.iter().enumerate() {
        let node_id = format!("ACT{}", i);
        lines.push(format!(
            "    {}{{\"{}\"}}",
            node_id,
            escape_mermaid(&action.action_type)
        ));
        lines.push(format!("    style {} fill:#fff9c4", node_id));
    }

    lines.push("".to_string());
    lines.push("    %% State Mutations".to_string());

    // Generate state mutation nodes
    for (i, mutation) in analysis.state_mutations.iter().enumerate() {
        let node_id = format!("STATE{}", i);
        let icon = mutation_icon(&mutation.mutation_type);
        lines.push(format!(
            "    {}([\"{} {}\"])",
            node_id,
            icon,
            escape_mermaid(&mutation.target)
        ));
        lines.push(format!("    style {} fill:#c8e6c9", node_id));
    }

    lines.push("".to_string());
    lines.push("    %% Connections".to_string());

    // Try to connect UI -> Action -> State based on context
    // This is a simple heuristic; more sophisticated analysis would track actual data flow
    connect_by_context(&mut lines, analysis);

    lines.join("\n")
}

/// Generate a summary Mermaid graph from multiple file analyses
pub fn generate_summary_mermaid(result: &AnalysisResult) -> String {
    let mut lines = vec![
        "flowchart LR".to_string(),
        "".to_string(),
        "    subgraph UI[\"UI Layer\"]".to_string(),
    ];

    // Group UI elements by type
    let mut ui_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for ui in result.all_ui_elements() {
        *ui_types.entry(ui.element_type.clone()).or_insert(0) += 1;
    }

    for (ui_type, count) in &ui_types {
        let icon = ui_icon(ui_type);
        lines.push(format!("        {}[\"{} {} ({})\"]", ui_type, icon, ui_type, count));
    }
    lines.push("    end".to_string());

    lines.push("".to_string());
    lines.push("    subgraph Actions[\"Action Layer\"]".to_string());

    // Group actions by type
    let mut action_types: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for action in result.all_actions() {
        *action_types.entry(action.action_type.clone()).or_insert(0) += 1;
    }

    for (action_type, count) in &action_types {
        lines.push(format!("        {}{{\"{}() ({})\"}}", action_type, action_type, count));
    }
    lines.push("    end".to_string());

    lines.push("".to_string());
    lines.push("    subgraph State[\"State Layer\"]".to_string());

    // Group state mutations by target (first component)
    let mut state_targets: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for mutation in result.all_state_mutations() {
        let key = mutation.target.split('.').take(2).collect::<Vec<_>>().join(".");
        *state_targets.entry(key).or_insert(0) += 1;
    }

    for (target, count) in &state_targets {
        let sanitized = target.replace('.', "_");
        lines.push(format!("        {}([\"{} ({})\"])", sanitized, target, count));
    }
    lines.push("    end".to_string());

    lines.push("".to_string());
    lines.push("    %% Layer connections".to_string());
    lines.push("    UI --> Actions".to_string());
    lines.push("    Actions --> State".to_string());

    lines.join("\n")
}

/// Connect nodes based on function context
fn connect_by_context(lines: &mut Vec<String>, analysis: &FileAnalysis) {
    // Group everything by context (function name)
    let mut by_context: std::collections::HashMap<String, (Vec<usize>, Vec<usize>, Vec<usize>)> =
        std::collections::HashMap::new();

    for (i, ui) in analysis.ui_elements.iter().enumerate() {
        by_context.entry(ui.context.clone()).or_default().0.push(i);
    }

    for (i, action) in analysis.actions.iter().enumerate() {
        by_context.entry(action.context.clone()).or_default().1.push(i);
    }

    for (i, mutation) in analysis.state_mutations.iter().enumerate() {
        by_context.entry(mutation.context.clone()).or_default().2.push(i);
    }

    // Create connections within each context
    for (context, (ui_indices, action_indices, state_indices)) in by_context {
        if context.is_empty() {
            continue;
        }

        // Simple heuristic: connect UI -> Action -> State in sequence
        for &ui_idx in &ui_indices {
            for &act_idx in &action_indices {
                lines.push(format!("    UI{} --> ACT{}", ui_idx, act_idx));
            }
        }

        for &act_idx in &action_indices {
            for &state_idx in &state_indices {
                lines.push(format!("    ACT{} --> STATE{}", act_idx, state_idx));
            }
        }
    }
}

/// Get an icon for UI element type
fn ui_icon(element_type: &str) -> &'static str {
    match element_type {
        "button" | "small_button" => "🔘",
        "label" | "heading" | "monospace" | "code" => "📝",
        "checkbox" | "toggle_value" => "☑️",
        "radio" | "radio_value" => "🔘",
        "text_edit_singleline" | "text_edit_multiline" => "✏️",
        "slider" | "drag_value" => "🎚️",
        "color_edit_button_rgb" | "color_edit_button_rgba" => "🎨",
        "image" => "🖼️",
        "hyperlink" | "hyperlink_to" => "🔗",
        "separator" => "➖",
        "spinner" | "progress_bar" => "⏳",
        "menu_button" | "collapsing" => "📂",
        _ => "📦",
    }
}

/// Get an icon for mutation type
fn mutation_icon(mutation_type: &str) -> &'static str {
    if mutation_type.starts_with("method:") {
        match mutation_type.strip_prefix("method:").unwrap_or("") {
            "push" | "insert" | "append" | "extend" => "➕",
            "pop" | "remove" | "clear" | "drain" => "➖",
            "toggle" => "🔄",
            _ => "📝",
        }
    } else {
        match mutation_type {
            "assign" => "=",
            "add_assign" => "+=",
            "sub_assign" => "-=",
            _ => "📝",
        }
    }
}

/// Escape special characters for Mermaid
fn escape_mermaid(s: &str) -> String {
    s.replace('"', "'")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('&', "&amp;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, StateMutation, UiElement};

    #[test]
    fn test_generate_simple_graph() {
        let analysis = FileAnalysis {
            path: "test.rs".to_string(),
            ui_elements: vec![UiElement {
                element_type: "button".to_string(),
                label: Some("Click me".to_string()),
                context: "show".to_string(),
                file_path: "test.rs".to_string(),
                line: 0,
                response_var: None,
            }],
            actions: vec![Action {
                action_type: "clicked".to_string(),
                source: "ui.button()".to_string(),
                context: "show".to_string(),
                file_path: "test.rs".to_string(),
                line: 0,
            }],
            state_mutations: vec![StateMutation {
                target: "state.counter".to_string(),
                mutation_type: "add_assign".to_string(),
                context: "show".to_string(),
                file_path: "test.rs".to_string(),
                line: 0,
            }],
            flows: Vec::new(),
        };

        let mermaid = generate_mermaid(&analysis);

        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("Click me"));
        assert!(mermaid.contains("clicked"));
        assert!(mermaid.contains("state.counter"));
    }
}
