//! egui-cha-analyzer: Static analyzer for egui UI flow
//!
//! Extracts and visualizes the UI -> Action -> State flow from egui code.
//!
//! # Example
//!
//! ```ignore
//! use egui_cha_analyzer::Analyzer;
//!
//! let analyzer = Analyzer::new();
//! let result = analyzer.analyze_file("src/app.rs")?;
//!
//! // Generate Mermaid flowchart
//! let mermaid = result.to_mermaid();
//! ```

pub mod action_extractor;
pub mod flow_extractor;
pub mod graph_generator;
pub mod state_extractor;
pub mod tea_extractor;
pub mod types;
pub mod ui_extractor;

use std::path::Path;
use types::{FileAnalysis, MsgEmission, MsgHandler, TeaFlow};

pub use types::AnalysisResult;

/// Build TEA flows by matching emissions to handlers
fn build_tea_flows(emissions: &[MsgEmission], handlers: &[MsgHandler]) -> Vec<TeaFlow> {
    emissions
        .iter()
        .map(|emission| {
            // Try to find a matching handler
            let handler = handlers.iter().find(|h| {
                // Match by message name (e.g., "Msg::Increment" matches "Msg::Increment")
                emission.msg == h.msg_pattern
                    || emission.msg.ends_with(&format!("::{}", h.msg_pattern))
                    || h.msg_pattern.ends_with(&format!(
                        "::{}",
                        emission.msg.split("::").last().unwrap_or("")
                    ))
            });

            TeaFlow {
                emission: emission.clone(),
                handler: handler.cloned(),
            }
        })
        .collect()
}

/// Main analyzer for egui UI flow
pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a single Rust source file
    pub fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Result<FileAnalysis, String> {
        let path = file_path.as_ref();
        let path_str = path.to_string_lossy().to_string();

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

        self.analyze_source(&path_str, &content)
    }

    /// Analyze source code directly
    pub fn analyze_source(&self, file_path: &str, content: &str) -> Result<FileAnalysis, String> {
        let syntax_tree =
            syn::parse_file(content).map_err(|e| format!("Parse error in {}: {}", file_path, e))?;

        // Extract UI elements (standard egui)
        let ui_elements = ui_extractor::extract_ui_elements(file_path, &syntax_tree);

        // Extract actions (response checks)
        let actions = action_extractor::extract_actions(file_path, &syntax_tree);

        // Extract state mutations
        let state_mutations = state_extractor::extract_state_mutations(file_path, &syntax_tree);

        // Extract flows with scope tracking (causality)
        let flows = flow_extractor::extract_flows(file_path, &syntax_tree);

        // Extract TEA patterns (DS components -> Msg)
        let msg_emissions = tea_extractor::extract_msg_emissions(file_path, &syntax_tree);

        // Extract Msg handlers (update function)
        let msg_handlers = tea_extractor::extract_msg_handlers(file_path, &syntax_tree);

        // Build TEA flows by matching emissions to handlers
        let tea_flows = build_tea_flows(&msg_emissions, &msg_handlers);

        Ok(FileAnalysis {
            path: file_path.to_string(),
            ui_elements,
            actions,
            state_mutations,
            flows,
            msg_emissions,
            msg_handlers,
            tea_flows,
        })
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_analysis() {
        let code = r#"
            fn show_ui(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.button("Click me").clicked() {
                    state.counter += 1;
                }
            }
        "#;

        let analyzer = Analyzer::new();
        let result = analyzer.analyze_source("test.rs", code).unwrap();

        assert!(!result.ui_elements.is_empty(), "Should find UI elements");
        assert!(!result.actions.is_empty(), "Should find actions");
        assert!(!result.flows.is_empty(), "Should find flows");

        // Check flow causality
        let flow = &result.flows[0];
        assert_eq!(flow.ui_element.element_type, "button");
        assert_eq!(flow.ui_element.label, Some("Click me".to_string()));
        assert_eq!(flow.action.action_type, "clicked");
        assert_eq!(flow.state_mutations[0].target, "state.counter");
    }
}
