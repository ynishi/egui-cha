//! Extract UI elements from egui code
//!
//! Detects patterns like:
//! - `ui.button("label")`
//! - `ui.label("text")`
//! - `ui.checkbox(&mut state, "label")`
//! - `ui.text_edit_singleline(&mut state.text)`

use crate::types::UiElement;
use syn::{visit::Visit, Expr, ExprMethodCall, File, Lit};

/// Known egui UI methods
const UI_METHODS: &[&str] = &[
    "button",
    "small_button",
    "label",
    "heading",
    "monospace",
    "code",
    "checkbox",
    "radio",
    "radio_value",
    "selectable_label",
    "selectable_value",
    "text_edit_singleline",
    "text_edit_multiline",
    "add",
    "add_sized",
    "slider",
    "drag_value",
    "color_edit_button_rgb",
    "color_edit_button_rgba",
    "image",
    "hyperlink",
    "hyperlink_to",
    "separator",
    "spinner",
    "progress_bar",
    "toggle_value",
    "collapsing",
    "menu_button",
];

/// Extract UI elements from a syntax tree
pub fn extract_ui_elements(file_path: &str, syntax_tree: &File) -> Vec<UiElement> {
    let mut visitor = UiVisitor {
        file_path: file_path.to_string(),
        elements: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.elements
}

struct UiVisitor {
    file_path: String,
    elements: Vec<UiElement>,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for UiVisitor {
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

        // Check if this is a UI method call
        if UI_METHODS.contains(&method_name.as_str()) {
            // Check if receiver looks like "ui"
            if is_ui_receiver(&node.receiver) {
                let label = extract_first_string_arg(&node.args);

                self.elements.push(UiElement {
                    element_type: method_name,
                    label,
                    context: self.current_function.clone().unwrap_or_default(),
                    file_path: self.file_path.clone(),
                    line: 0,
                    response_var: None,
                });
            }
        }

        // Continue visiting
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Check if the receiver expression looks like a UI reference
fn is_ui_receiver(expr: &Expr) -> bool {
    match expr {
        Expr::Path(path) => {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();
                return name == "ui" || name.ends_with("_ui") || name.contains("ui");
            }
            false
        }
        Expr::Reference(ref_expr) => is_ui_receiver(&ref_expr.expr),
        Expr::MethodCall(call) => {
            // Could be something like ctx.ui() or frame.ui()
            let method = call.method.to_string();
            method == "ui" || is_ui_receiver(&call.receiver)
        }
        _ => false,
    }
}

/// Extract the first string literal argument from a method call
fn extract_first_string_arg(args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>) -> Option<String> {
    for arg in args {
        if let Some(s) = extract_string_from_expr(arg) {
            return Some(s);
        }
    }
    None
}

/// Extract string literal from an expression
fn extract_string_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(expr_lit) => {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                return Some(lit_str.value());
            }
            None
        }
        Expr::Reference(ref_expr) => extract_string_from_expr(&ref_expr.expr),
        Expr::Call(_) => {
            // Handle format!() or similar - just return None for now
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_button() {
        let code = r#"
            fn show(ui: &mut egui::Ui) {
                ui.button("Click me");
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let elements = extract_ui_elements("test.rs", &syntax_tree);

        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "button");
        assert_eq!(elements[0].label, Some("Click me".to_string()));
        assert_eq!(elements[0].context, "show");
    }

    #[test]
    fn test_extract_multiple_elements() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut State) {
                ui.heading("Title");
                ui.label("Description");
                ui.checkbox(&mut state.enabled, "Enable");
                ui.button("Submit");
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let elements = extract_ui_elements("test.rs", &syntax_tree);

        assert_eq!(elements.len(), 4);
        assert_eq!(elements[0].element_type, "heading");
        assert_eq!(elements[1].element_type, "label");
        assert_eq!(elements[2].element_type, "checkbox");
        assert_eq!(elements[3].element_type, "button");
    }

    #[test]
    fn test_nested_ui() {
        let code = r#"
            fn show(ctx: &egui::Context) {
                egui::Window::new("Test").show(ctx, |ui| {
                    ui.button("Inside window");
                });
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let elements = extract_ui_elements("test.rs", &syntax_tree);

        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].element_type, "button");
        assert_eq!(elements[0].label, Some("Inside window".to_string()));
    }
}
