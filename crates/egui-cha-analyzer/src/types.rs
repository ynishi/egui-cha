//! Core types for egui flow analysis

/// A UI element found in the code (e.g., button, label, checkbox)
#[derive(Debug, Clone, PartialEq)]
pub struct UiElement {
    /// Type of UI element (button, label, checkbox, etc.)
    pub element_type: String,
    /// Label or identifier if available
    pub label: Option<String>,
    /// Function/method containing this element
    pub context: String,
    /// File path
    pub file_path: String,
    /// Line number (0 if unknown)
    pub line: usize,
    /// Variable name if response is stored
    pub response_var: Option<String>,
}

/// An action triggered by UI interaction
#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    /// Type of action (clicked, changed, dragged, etc.)
    pub action_type: String,
    /// The UI element or response variable this action checks
    pub source: String,
    /// Function/method containing this action
    pub context: String,
    /// File path
    pub file_path: String,
    /// Line number (0 if unknown)
    pub line: usize,
}

/// A state mutation
#[derive(Debug, Clone, PartialEq)]
pub struct StateMutation {
    /// The state field being mutated
    pub target: String,
    /// Type of mutation (assign, increment, method call, etc.)
    pub mutation_type: String,
    /// Function/method containing this mutation
    pub context: String,
    /// File path
    pub file_path: String,
    /// Line number (0 if unknown)
    pub line: usize,
}

/// A complete UI flow: UI element -> Action -> State mutations
/// Represents a causal chain from user interaction to state changes
#[derive(Debug, Clone)]
pub struct UiFlow {
    pub ui_element: UiElement,
    pub action: Action,
    pub state_mutations: Vec<StateMutation>,
    pub context: String,
}

/// Analysis result for a single file
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: String,
    pub ui_elements: Vec<UiElement>,
    pub actions: Vec<Action>,
    pub state_mutations: Vec<StateMutation>,
    /// Scope-aware flows (UI -> Action -> State with causality)
    pub flows: Vec<UiFlow>,
}

impl FileAnalysis {
    pub fn new(path: String) -> Self {
        Self {
            path,
            ui_elements: Vec::new(),
            actions: Vec::new(),
            state_mutations: Vec::new(),
            flows: Vec::new(),
        }
    }
}

/// Aggregated analysis result for multiple files
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub files: Vec<FileAnalysis>,
}

impl AnalysisResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, analysis: FileAnalysis) {
        self.files.push(analysis);
    }

    pub fn all_ui_elements(&self) -> impl Iterator<Item = &UiElement> {
        self.files.iter().flat_map(|f| &f.ui_elements)
    }

    pub fn all_actions(&self) -> impl Iterator<Item = &Action> {
        self.files.iter().flat_map(|f| &f.actions)
    }

    pub fn all_state_mutations(&self) -> impl Iterator<Item = &StateMutation> {
        self.files.iter().flat_map(|f| &f.state_mutations)
    }
}
