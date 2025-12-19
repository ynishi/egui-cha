//! Extract UI flows with scope-aware analysis
//!
//! Tracks causality: `if ui.button("x").clicked() { state.y = z }`
//! -> UiFlow { ui: button("x"), action: clicked, mutations: [state.y = z] }
//!
//! Also tracks response variable bindings:
//! `let r = ui.button("x"); if r.clicked() { ... }`

use crate::types::{Action, StateMutation, UiElement, UiFlow};
use std::collections::HashMap;
use syn::{
    visit::Visit, BinOp, Expr, ExprBinary, ExprIf, ExprMethodCall, File, Lit,
    Pat, Stmt,
};

/// Extract UI flows with scope tracking
pub fn extract_flows(file_path: &str, syntax_tree: &File) -> Vec<UiFlow> {
    let mut visitor = FlowVisitor {
        file_path: file_path.to_string(),
        flows: Vec::new(),
        current_function: None,
        response_bindings: HashMap::new(),
    };

    visitor.visit_file(syntax_tree);
    visitor.flows
}

struct FlowVisitor {
    file_path: String,
    flows: Vec<UiFlow>,
    current_function: Option<String>,
    /// Maps variable names to their UI element bindings
    /// e.g., "r" -> UiElement { type: "button", label: "Test" }
    response_bindings: HashMap<String, UiElement>,
}

impl<'ast> Visit<'ast> for FlowVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let old_function = self.current_function.clone();
        let old_bindings = std::mem::take(&mut self.response_bindings);

        self.current_function = Some(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);

        self.current_function = old_function;
        self.response_bindings = old_bindings;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let old_function = self.current_function.clone();
        let old_bindings = std::mem::take(&mut self.response_bindings);

        self.current_function = Some(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);

        self.current_function = old_function;
        self.response_bindings = old_bindings;
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        // Check for `let var = ui.xxx()` bindings
        if let Stmt::Local(local) = node {
            if let Some(init) = &local.init {
                if let Some(ui_element) = try_extract_ui_from_expr(&init.expr, &self.file_path, &self.current_function) {
                    // Get variable name from pattern
                    if let Pat::Ident(pat_ident) = &local.pat {
                        let var_name = pat_ident.ident.to_string();
                        self.response_bindings.insert(var_name, ui_element);
                    }
                }
            }
        }

        syn::visit::visit_stmt(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        // Extract triggers from condition (UI + Action pairs)
        let triggers = extract_triggers_from_condition(
            &node.cond,
            &self.file_path,
            &self.current_function,
            &self.response_bindings,
        );

        // Extract state mutations from then block
        let mutations = extract_mutations_from_block(&node.then_branch, &self.file_path, &self.current_function);

        // Create flows for each trigger-mutation pair
        for trigger in &triggers {
            if !mutations.is_empty() {
                self.flows.push(UiFlow {
                    ui_element: trigger.0.clone(),
                    action: trigger.1.clone(),
                    state_mutations: mutations.clone(),
                    context: self.current_function.clone().unwrap_or_default(),
                });
            }
        }

        // Continue visiting nested ifs
        syn::visit::visit_expr_if(self, node);
    }
}

/// Try to extract UI element from an expression (for let bindings)
fn try_extract_ui_from_expr(
    expr: &Expr,
    file_path: &str,
    current_function: &Option<String>,
) -> Option<UiElement> {
    match expr {
        Expr::MethodCall(call) => {
            let method_name = call.method.to_string();

            if UI_METHODS.contains(&method_name.as_str()) {
                let label = extract_first_string_arg(&call.args);
                return Some(UiElement {
                    element_type: method_name,
                    label,
                    context: current_function.clone().unwrap_or_default(),
                    file_path: file_path.to_string(),
                    line: 0,
                    response_var: None,
                });
            }

            // Recurse into receiver for chained calls
            try_extract_ui_from_expr(&call.receiver, file_path, current_function)
        }
        Expr::Paren(paren) => try_extract_ui_from_expr(&paren.expr, file_path, current_function),
        _ => None,
    }
}

/// Extract (UiElement, Action) pairs from an if condition
fn extract_triggers_from_condition(
    expr: &Expr,
    file_path: &str,
    current_function: &Option<String>,
    response_bindings: &HashMap<String, UiElement>,
) -> Vec<(UiElement, Action)> {
    let mut triggers = Vec::new();
    collect_triggers(expr, file_path, current_function, response_bindings, &mut triggers);
    triggers
}

fn collect_triggers(
    expr: &Expr,
    file_path: &str,
    current_function: &Option<String>,
    response_bindings: &HashMap<String, UiElement>,
    triggers: &mut Vec<(UiElement, Action)>,
) {
    match expr {
        // Handle || (Or) - multiple conditions
        Expr::Binary(ExprBinary { left, op: BinOp::Or(_), right, .. }) => {
            collect_triggers(left, file_path, current_function, response_bindings, triggers);
            collect_triggers(right, file_path, current_function, response_bindings, triggers);
        }
        // Handle && (And) - just recurse
        Expr::Binary(ExprBinary { left, op: BinOp::And(_), right, .. }) => {
            collect_triggers(left, file_path, current_function, response_bindings, triggers);
            collect_triggers(right, file_path, current_function, response_bindings, triggers);
        }
        // Handle method call like `.clicked()`, `.changed()`
        Expr::MethodCall(call) => {
            if let Some(trigger) = extract_trigger_from_method_call(call, file_path, current_function, response_bindings) {
                triggers.push(trigger);
            }
        }
        // Handle parenthesized expression
        Expr::Paren(paren) => {
            collect_triggers(&paren.expr, file_path, current_function, response_bindings, triggers);
        }
        _ => {}
    }
}

/// Known action methods
const ACTION_METHODS: &[&str] = &[
    "clicked", "clicked_by", "secondary_clicked", "middle_clicked",
    "double_clicked", "triple_clicked", "changed", "dragged",
    "drag_started", "drag_stopped", "hovered", "has_focus",
    "gained_focus", "lost_focus",
];

/// Extract a trigger (UiElement + Action) from a method call chain
fn extract_trigger_from_method_call(
    call: &ExprMethodCall,
    file_path: &str,
    current_function: &Option<String>,
    response_bindings: &HashMap<String, UiElement>,
) -> Option<(UiElement, Action)> {
    let method_name = call.method.to_string();

    // Is this an action method?
    if !ACTION_METHODS.contains(&method_name.as_str()) {
        return None;
    }

    let action = Action {
        action_type: method_name,
        source: describe_expr(&call.receiver),
        context: current_function.clone().unwrap_or_default(),
        file_path: file_path.to_string(),
        line: 0,
    };

    // Try to find the UI element in the receiver chain (with variable resolution)
    let ui_element = extract_ui_from_chain(&call.receiver, file_path, current_function, response_bindings);

    Some((ui_element, action))
}

/// Known UI methods
const UI_METHODS: &[&str] = &[
    "button", "small_button", "label", "heading", "checkbox", "radio",
    "radio_value", "selectable_label", "selectable_value",
    "text_edit_singleline", "text_edit_multiline", "slider", "drag_value",
    "toggle_value", "menu_button", "collapsing", "add",
];

/// Extract UI element from a method call chain (e.g., `ui.button("x")`)
/// Also resolves response variable bindings (e.g., `let r = ui.button(); r.clicked()`)
fn extract_ui_from_chain(
    expr: &Expr,
    file_path: &str,
    current_function: &Option<String>,
    response_bindings: &HashMap<String, UiElement>,
) -> UiElement {
    match expr {
        Expr::MethodCall(call) => {
            let method_name = call.method.to_string();

            if UI_METHODS.contains(&method_name.as_str()) {
                // Found UI method
                let label = extract_first_string_arg(&call.args);

                return UiElement {
                    element_type: method_name,
                    label,
                    context: current_function.clone().unwrap_or_default(),
                    file_path: file_path.to_string(),
                    line: 0,
                    response_var: None,
                };
            }

            // Recurse into receiver
            extract_ui_from_chain(&call.receiver, file_path, current_function, response_bindings)
        }
        Expr::Path(path) => {
            // Variable reference (e.g., `response`)
            let var_name = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Try to resolve from bindings first
            if let Some(bound_ui) = response_bindings.get(&var_name) {
                // Return the bound UI element with the variable name attached
                return UiElement {
                    element_type: bound_ui.element_type.clone(),
                    label: bound_ui.label.clone(),
                    context: current_function.clone().unwrap_or_default(),
                    file_path: file_path.to_string(),
                    line: 0,
                    response_var: Some(var_name),
                };
            }

            // Fallback: unknown response variable
            UiElement {
                element_type: "response_var".to_string(),
                label: Some(var_name.clone()),
                context: current_function.clone().unwrap_or_default(),
                file_path: file_path.to_string(),
                line: 0,
                response_var: Some(var_name),
            }
        }
        _ => UiElement {
            element_type: "unknown".to_string(),
            label: None,
            context: current_function.clone().unwrap_or_default(),
            file_path: file_path.to_string(),
            line: 0,
            response_var: None,
        },
    }
}

/// Extract state mutations from a block
fn extract_mutations_from_block(
    block: &syn::Block,
    file_path: &str,
    current_function: &Option<String>,
) -> Vec<StateMutation> {
    let mut mutations = Vec::new();
    let mut visitor = MutationVisitor {
        file_path: file_path.to_string(),
        current_function: current_function.clone(),
        mutations: &mut mutations,
    };

    for stmt in &block.stmts {
        visitor.visit_stmt(stmt);
    }

    mutations
}

struct MutationVisitor<'a> {
    file_path: String,
    current_function: Option<String>,
    mutations: &'a mut Vec<StateMutation>,
}

impl<'ast, 'a> Visit<'ast> for MutationVisitor<'a> {
    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        let target = describe_expr(&node.left);

        if is_likely_state_mutation(&target) {
            self.mutations.push(StateMutation {
                target,
                mutation_type: "assign".to_string(),
                context: self.current_function.clone().unwrap_or_default(),
                file_path: self.file_path.clone(),
                line: 0,
            });
        }

        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        let op_str = match &node.op {
            BinOp::AddAssign(_) => Some("add_assign"),
            BinOp::SubAssign(_) => Some("sub_assign"),
            BinOp::MulAssign(_) => Some("mul_assign"),
            BinOp::DivAssign(_) => Some("div_assign"),
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

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();
        let mutating_methods = [
            "push", "pop", "insert", "remove", "clear", "append", "extend",
            "retain", "drain", "truncate", "toggle", "set", "take", "replace",
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

    // Don't recurse into nested if expressions (those will be handled separately)
    fn visit_expr_if(&mut self, _node: &'ast ExprIf) {
        // Skip - nested ifs are handled by the main FlowVisitor
    }
}

fn is_likely_state_mutation(target: &str) -> bool {
    target.contains('.')
        || target.starts_with("self.")
        || target.contains("state")
        || target.starts_with('*')
}

fn describe_expr(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => {
            path.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        }
        Expr::Field(field) => {
            let base = describe_expr(&field.base);
            match &field.member {
                syn::Member::Named(ident) => format!("{}.{}", base, ident),
                syn::Member::Unnamed(idx) => format!("{}.{}", base, idx.index),
            }
        }
        Expr::MethodCall(call) => {
            let receiver = describe_expr(&call.receiver);
            let method = call.method.to_string();
            format!("{}.{}()", receiver, method)
        }
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Deref(_)) => {
            format!("*{}", describe_expr(&unary.expr))
        }
        Expr::Reference(ref_expr) => describe_expr(&ref_expr.expr),
        Expr::Paren(paren) => describe_expr(&paren.expr),
        Expr::Index(idx) => {
            let base = describe_expr(&idx.expr);
            format!("{}[..]", base)
        }
        _ => "<expr>".to_string(),
    }
}

fn extract_first_string_arg(
    args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>,
) -> Option<String> {
    for arg in args {
        if let Some(s) = extract_string_from_expr(arg) {
            return Some(s);
        }
    }
    None
}

fn extract_string_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(expr_lit) => {
            if let Lit::Str(lit_str) = &expr_lit.lit {
                return Some(lit_str.value());
            }
            None
        }
        Expr::Reference(ref_expr) => extract_string_from_expr(&ref_expr.expr),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_button_click_flow() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.button("Click").clicked() {
                    state.counter += 1;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].ui_element.element_type, "button");
        assert_eq!(flows[0].ui_element.label, Some("Click".to_string()));
        assert_eq!(flows[0].action.action_type, "clicked");
        assert_eq!(flows[0].state_mutations.len(), 1);
        assert_eq!(flows[0].state_mutations[0].target, "state.counter");
    }

    #[test]
    fn test_multiple_mutations_in_block() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.button("Reset").clicked() {
                    state.counter = 0;
                    state.name = String::new();
                    state.enabled = false;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].state_mutations.len(), 3);
    }

    #[test]
    fn test_multiple_buttons() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.button("-").clicked() {
                    state.counter -= 1;
                }
                if ui.button("+").clicked() {
                    state.counter += 1;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 2);
        assert_eq!(flows[0].ui_element.label, Some("-".to_string()));
        assert_eq!(flows[0].state_mutations[0].mutation_type, "sub_assign");
        assert_eq!(flows[1].ui_element.label, Some("+".to_string()));
        assert_eq!(flows[1].state_mutations[0].mutation_type, "add_assign");
    }

    #[test]
    fn test_or_condition() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                let r = ui.button("Test");
                if r.clicked() || r.secondary_clicked() {
                    state.activated = true;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        // Should create 2 flows (one for each trigger)
        assert_eq!(flows.len(), 2);
        assert_eq!(flows[0].action.action_type, "clicked");
        assert_eq!(flows[1].action.action_type, "secondary_clicked");
    }

    #[test]
    fn test_checkbox_changed() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.checkbox(&mut state.enabled, "Enable").changed() {
                    state.settings_dirty = true;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].ui_element.element_type, "checkbox");
        assert_eq!(flows[0].action.action_type, "changed");
    }

    #[test]
    fn test_method_mutation() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                if ui.button("Add").clicked() {
                    state.items.push("item".to_string());
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].state_mutations[0].mutation_type, "method:push");
    }

    #[test]
    fn test_response_variable_resolution() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                let response = ui.button("Save");
                if response.clicked() {
                    state.saved = true;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 1);
        // Should resolve 'response' to the original button
        assert_eq!(flows[0].ui_element.element_type, "button");
        assert_eq!(flows[0].ui_element.label, Some("Save".to_string()));
        assert_eq!(flows[0].ui_element.response_var, Some("response".to_string()));
        assert_eq!(flows[0].action.action_type, "clicked");
    }

    #[test]
    fn test_response_variable_multiple_uses() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                let btn = ui.button("Action");
                if btn.clicked() {
                    state.action_count += 1;
                }
                if btn.hovered() {
                    state.hover_count += 1;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 2);
        // Both flows should resolve to the same button
        assert_eq!(flows[0].ui_element.element_type, "button");
        assert_eq!(flows[0].ui_element.label, Some("Action".to_string()));
        assert_eq!(flows[0].action.action_type, "clicked");

        assert_eq!(flows[1].ui_element.element_type, "button");
        assert_eq!(flows[1].ui_element.label, Some("Action".to_string()));
        assert_eq!(flows[1].action.action_type, "hovered");
    }

    #[test]
    fn test_response_var_with_or_condition() {
        let code = r#"
            fn show(ui: &mut egui::Ui, state: &mut AppState) {
                let r = ui.button("Multi");
                if r.clicked() || r.secondary_clicked() {
                    state.activated = true;
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let flows = extract_flows("test.rs", &syntax_tree);

        assert_eq!(flows.len(), 2);
        // Both should resolve to "Multi" button
        assert_eq!(flows[0].ui_element.label, Some("Multi".to_string()));
        assert_eq!(flows[1].ui_element.label, Some("Multi".to_string()));
        assert_eq!(flows[0].action.action_type, "clicked");
        assert_eq!(flows[1].action.action_type, "secondary_clicked");
    }
}
