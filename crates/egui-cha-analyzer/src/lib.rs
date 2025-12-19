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

pub mod types;
pub mod ui_extractor;
pub mod action_extractor;
pub mod state_extractor;
pub mod flow_extractor;
pub mod graph_generator;

use std::path::Path;
use types::FileAnalysis;

pub use types::AnalysisResult;

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
        let syntax_tree = syn::parse_file(content)
            .map_err(|e| format!("Parse error in {}: {}", file_path, e))?;

        // Extract UI elements
        let ui_elements = ui_extractor::extract_ui_elements(file_path, &syntax_tree);

        // Extract actions (response checks)
        let actions = action_extractor::extract_actions(file_path, &syntax_tree);

        // Extract state mutations
        let state_mutations = state_extractor::extract_state_mutations(file_path, &syntax_tree);

        // Extract flows with scope tracking (causality)
        let flows = flow_extractor::extract_flows(file_path, &syntax_tree);

        Ok(FileAnalysis {
            path: file_path.to_string(),
            ui_elements,
            actions,
            state_mutations,
            flows,
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
