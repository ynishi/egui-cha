//! Extract state mutations from egui code
//!
//! Detects patterns like:
//! - `state.field = value`
//! - `state.counter += 1`
//! - `*value = new_value`
//! - `state.list.push(item)`

use crate::types::StateMutation;
use syn::{visit::Visit, BinOp, Expr, File};

/// Extract state mutations from a syntax tree
pub fn extract_state_mutations(file_path: &str, syntax_tree: &File) -> Vec<StateMutation> {
    let mut visitor = StateVisitor {
        file_path: file_path.to_string(),
        mutations: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.mutations
}

struct StateVisitor {
    file_path: String,
    mutations: Vec<StateMutation>,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for StateVisitor {
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

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        let target = describe_expr(&node.left);

        // Skip if it looks like a local variable (not a state field)
        if !is_likely_state_mutation(&target) {
            syn::visit::visit_expr_assign(self, node);
            return;
        }

        self.mutations.push(StateMutation {
            target,
            mutation_type: "assign".to_string(),
            context: self.current_function.clone().unwrap_or_default(),
            file_path: self.file_path.clone(),
            line: 0,
        });

        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        // Check for compound assignment operators (+=, -=, etc.)
        let op_str = match &node.op {
            BinOp::AddAssign(_) => Some("add_assign"),
            BinOp::SubAssign(_) => Some("sub_assign"),
            BinOp::MulAssign(_) => Some("mul_assign"),
            BinOp::DivAssign(_) => Some("div_assign"),
            BinOp::RemAssign(_) => Some("rem_assign"),
            BinOp::BitAndAssign(_) => Some("bitand_assign"),
            BinOp::BitOrAssign(_) => Some("bitor_assign"),
            BinOp::BitXorAssign(_) => Some("bitxor_assign"),
            BinOp::ShlAssign(_) => Some("shl_assign"),
            BinOp::ShrAssign(_) => Some("shr_assign"),
            _ => None,
        };

        if let Some(mutation_type) = op_str {
            let target = describe_expr(&node.left);

            if is_likely_state_mutation(&target) {
                self.mutations.push(StateMutation {
                    target,
                    mutation_type: mutation_type.to_string(),
                    context: self.current_function.clone().unwrap_or_default(),
                    file_path: self.file_path.clone(),
                    line: 0,
                });
            }
        }

        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        // Check for mutating method calls like .push(), .pop(), .clear(), etc.
        let method = node.method.to_string();
        let mutating_methods = [
            "push",
            "pop",
            "insert",
            "remove",
            "clear",
            "append",
            "extend",
            "retain",
            "drain",
            "truncate",
            "swap_remove",
            "set",
            "take",
            "replace",
            "get_or_insert",
            "toggle",
        ];

        if mutating_methods.contains(&method.as_str()) {
            let receiver = describe_expr(&node.receiver);

            if is_likely_state_mutation(&receiver) {
                self.mutations.push(StateMutation {
                    target: receiver,
                    mutation_type: format!("method:{}", method),
                    context: self.current_function.clone().unwrap_or_default(),
                    file_path: self.file_path.clone(),
                    line: 0,
                });
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Check if this looks like a state mutation (not a local variable)
fn is_likely_state_mutation(target: &str) -> bool {
    // State mutations typically have a dot (field access)
    // or contain "state", "self", etc.
    target.contains('.')
        || target.starts_with("self.")
        || target.contains("state")
        || target.starts_with('*')
}

/// Describe an expression for documentation
fn describe_expr(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        Expr::Field(field) => {
            let base = describe_expr(&field.base);
            match &field.member {
                syn::Member::Named(ident) => format!("{}.{}", base, ident),
                syn::Member::Unnamed(idx) => format!("{}.{}", base, idx.index),
            }
        }
        Expr::Index(idx) => {
            let base = describe_expr(&idx.expr);
            format!("{}[..]", base)
        }
        Expr::Unary(unary) => {
            if matches!(unary.op, syn::UnOp::Deref(_)) {
                format!("*{}", describe_expr(&unary.expr))
            } else {
                describe_expr(&unary.expr)
            }
        }
        Expr::Reference(ref_expr) => describe_expr(&ref_expr.expr),
        Expr::Paren(paren) => describe_expr(&paren.expr),
        Expr::MethodCall(call) => {
            let receiver = describe_expr(&call.receiver);
            let method = call.method.to_string();
            format!("{}.{}()", receiver, method)
        }
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_field_assign() {
        let code = r#"
            fn update(state: &mut AppState) {
                state.counter = 0;
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let mutations = extract_state_mutations("test.rs", &syntax_tree);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].target, "state.counter");
        assert_eq!(mutations[0].mutation_type, "assign");
    }

    #[test]
    fn test_extract_add_assign() {
        let code = r#"
            fn increment(state: &mut AppState) {
                state.counter += 1;
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let mutations = extract_state_mutations("test.rs", &syntax_tree);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].target, "state.counter");
        assert_eq!(mutations[0].mutation_type, "add_assign");
    }

    #[test]
    fn test_extract_method_mutation() {
        let code = r#"
            fn add_item(state: &mut AppState, item: Item) {
                state.items.push(item);
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let mutations = extract_state_mutations("test.rs", &syntax_tree);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].target, "state.items");
        assert_eq!(mutations[0].mutation_type, "method:push");
    }

    #[test]
    fn test_extract_nested_field() {
        let code = r#"
            fn update_name(state: &mut AppState) {
                state.user.profile.name = "New Name".to_string();
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let mutations = extract_state_mutations("test.rs", &syntax_tree);

        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0].target, "state.user.profile.name");
    }

    #[test]
    fn test_ignore_local_variable() {
        let code = r#"
            fn calculate() {
                let x = 5;
                let mut y = 10;
                y = 20;
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let mutations = extract_state_mutations("test.rs", &syntax_tree);

        // Should not detect local variable mutations
        assert!(mutations.is_empty());
    }

    #[test]
    fn test_full_ui_flow() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.button("Increment").clicked() {
                    state.counter += 1;
                }
                if ui.button("Reset").clicked() {
                    state.counter = 0;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let mutations = extract_state_mutations("test.rs", &syntax_tree);

        assert_eq!(mutations.len(), 2);
    }
}
