//! Extract actions from egui Response checks
//!
//! Detects patterns like:
//! - `response.clicked()`
//! - `ui.button("x").clicked()`
//! - `response.changed()`
//! - `response.dragged()`

use crate::types::Action;
use syn::{visit::Visit, Expr, ExprMethodCall, File};

/// Known egui Response action methods
const ACTION_METHODS: &[&str] = &[
    "clicked",
    "clicked_by",
    "secondary_clicked",
    "middle_clicked",
    "double_clicked",
    "triple_clicked",
    "changed",
    "dragged",
    "drag_started",
    "drag_stopped",
    "hovered",
    "highlighted",
    "has_focus",
    "gained_focus",
    "lost_focus",
    "enabled",
    "clicked_elsewhere",
    "is_pointer_button_down_on",
];

/// Extract actions from a syntax tree
pub fn extract_actions(file_path: &str, syntax_tree: &File) -> Vec<Action> {
    let mut visitor = ActionVisitor {
        file_path: file_path.to_string(),
        actions: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.actions
}

struct ActionVisitor {
    file_path: String,
    actions: Vec<Action>,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for ActionVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let old = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.current_function = old;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let old = self.current_function.clone();
        self.current_function = Some(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.current_function = old;
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_string();

        // Check if this is an action method
        if ACTION_METHODS.contains(&method_name.as_str()) {
            let source = describe_receiver(&node.receiver);

            self.actions.push(Action {
                action_type: method_name,
                source,
                context: self.current_function.clone().unwrap_or_default(),
                file_path: self.file_path.clone(),
                line: 0,
            });
        }

        // Continue visiting
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Describe the receiver expression for documentation
fn describe_receiver(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        Expr::MethodCall(call) => {
            let receiver = describe_receiver(&call.receiver);
            let method = call.method.to_string();
            format!("{}.{}()", receiver, method)
        }
        Expr::Field(field) => {
            let base = describe_receiver(&field.base);
            match &field.member {
                syn::Member::Named(ident) => format!("{}.{}", base, ident),
                syn::Member::Unnamed(idx) => format!("{}.{}", base, idx.index),
            }
        }
        Expr::Reference(ref_expr) => describe_receiver(&ref_expr.expr),
        Expr::Paren(paren) => describe_receiver(&paren.expr),
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_clicked() {
        let code = r#"
            fn show(ui: &mut egui::Ui) {
                if ui.button("Click").clicked() {
                    println!("clicked!");
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let actions = extract_actions("test.rs", &syntax_tree);

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, "clicked");
        assert!(actions[0].source.contains("button"));
    }

    #[test]
    fn test_extract_response_variable() {
        let code = r#"
            fn show(ui: &mut egui::Ui) {
                let response = ui.button("Click");
                if response.clicked() {
                    // do something
                }
                if response.hovered() {
                    // highlight
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let actions = extract_actions("test.rs", &syntax_tree);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].action_type, "clicked");
        assert_eq!(actions[0].source, "response");
        assert_eq!(actions[1].action_type, "hovered");
        assert_eq!(actions[1].source, "response");
    }

    #[test]
    fn test_extract_changed() {
        let code = r#"
            fn show(ui: &mut egui::Ui, value: &mut f32) {
                if ui.slider(value, 0.0..=100.0).changed() {
                    println!("slider changed");
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let actions = extract_actions("test.rs", &syntax_tree);

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, "changed");
    }

    #[test]
    fn test_multiple_actions_chained() {
        let code = r#"
            fn show(ui: &mut egui::Ui) {
                let r = ui.button("Test");
                if r.clicked() || r.secondary_clicked() {
                    // handle
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let actions = extract_actions("test.rs", &syntax_tree);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].action_type, "clicked");
        assert_eq!(actions[1].action_type, "secondary_clicked");
    }
}
