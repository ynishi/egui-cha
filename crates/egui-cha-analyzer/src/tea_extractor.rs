//! Extract TEA (The Elm Architecture) patterns
//!
//! Detects:
//! - DS component -> Msg: `Button::primary("+").on_click(ctx, Msg::Increment)`
//! - Msg -> State: `Msg::Increment => model.counter += 1` in update function

use crate::types::{MsgEmission, MsgHandler, StateMutation};
use syn::{
    visit::Visit, Arm, BinOp, Expr, ExprMatch, ExprMethodCall, File, Pat,
};

// ============================================================
// DS Component -> Msg Extraction
// ============================================================

/// Known DS component types
const DS_COMPONENTS: &[&str] = &["Button", "Input", "Card", "Badge", "Icon"];

/// Known DS action methods that emit messages
const DS_ACTIONS: &[&str] = &["on_click", "on_change", "show_with"];

/// Extract MsgEmissions from DS component patterns
pub fn extract_msg_emissions(file_path: &str, syntax_tree: &File) -> Vec<MsgEmission> {
    let mut visitor = EmissionVisitor {
        file_path: file_path.to_string(),
        emissions: Vec::new(),
        current_function: None,
    };

    visitor.visit_file(syntax_tree);
    visitor.emissions
}

struct EmissionVisitor {
    file_path: String,
    emissions: Vec<MsgEmission>,
    current_function: Option<String>,
}

impl<'ast> Visit<'ast> for EmissionVisitor {
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

        // Check for DS action methods
        if DS_ACTIONS.contains(&method_name.as_str()) {
            if let Some(emission) = try_extract_emission(node, &self.file_path, &self.current_function) {
                self.emissions.push(emission);
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Try to extract a MsgEmission from a method call chain
fn try_extract_emission(
    call: &ExprMethodCall,
    file_path: &str,
    current_function: &Option<String>,
) -> Option<MsgEmission> {
    let action = call.method.to_string();

    // Extract the message argument
    // Pattern: .on_click(ctx, Msg::Variant) or .on_click(ctx, Msg::Variant(args))
    let msg = extract_msg_arg(&call.args)?;

    // Walk up the receiver chain to find the DS component
    let (component, variant, label) = extract_ds_component(&call.receiver)?;

    Some(MsgEmission {
        component,
        variant,
        label,
        action,
        msg,
        context: current_function.clone().unwrap_or_default(),
        file_path: file_path.to_string(),
    })
}

/// Extract the Msg argument from method args
fn extract_msg_arg(args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>) -> Option<String> {
    // Usually the message is the second argument (after ctx)
    // .on_click(ctx, Msg::Increment)
    for arg in args.iter().skip(1) {
        let msg_str = expr_to_msg_string(arg);
        if msg_str.contains("::") || msg_str.starts_with("Msg") {
            return Some(msg_str);
        }
    }
    // Try first arg if no ctx
    if let Some(first) = args.first() {
        let msg_str = expr_to_msg_string(first);
        if msg_str.contains("::") || msg_str.starts_with("Msg") {
            return Some(msg_str);
        }
    }
    None
}

/// Convert an expression to a message string representation
fn expr_to_msg_string(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => {
            path.path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
        }
        Expr::Call(call) => {
            // Msg::Variant(args)
            if let Expr::Path(path) = &*call.func {
                path.path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::")
            } else {
                "<call>".to_string()
            }
        }
        _ => "<expr>".to_string(),
    }
}

/// Extract DS component info from receiver chain
/// Returns (component, variant, label)
fn extract_ds_component(expr: &Expr) -> Option<(String, String, Option<String>)> {
    match expr {
        Expr::MethodCall(call) => {
            // Check if this is a variant method (primary, secondary, ghost, etc.)
            let method = call.method.to_string();

            // Recurse to find the actual DS component
            if let Expr::Path(path) = &*call.receiver {
                // Button::primary("label")
                let component = path.path.segments.last()?.ident.to_string();
                if DS_COMPONENTS.contains(&component.as_str()) {
                    let label = extract_first_string_arg(&call.args);
                    return Some((component, method, label));
                }
            }

            // Try to recurse further (for chained calls)
            extract_ds_component(&call.receiver)
        }
        Expr::Call(call) => {
            // Button::new("label") style
            if let Expr::Path(path) = &*call.func {
                let segments: Vec<_> = path.path.segments.iter().map(|s| s.ident.to_string()).collect();
                if segments.len() >= 2 {
                    let component = &segments[segments.len() - 2];
                    let variant = &segments[segments.len() - 1];
                    if DS_COMPONENTS.contains(&component.as_str()) {
                        let label = extract_first_string_arg(&call.args);
                        return Some((component.clone(), variant.clone(), label));
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_first_string_arg(args: &syn::punctuated::Punctuated<Expr, syn::token::Comma>) -> Option<String> {
    for arg in args {
        if let Expr::Lit(expr_lit) = arg {
            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                return Some(lit_str.value());
            }
        }
    }
    None
}

// ============================================================
// Msg -> State Extraction (update function)
// ============================================================

/// Extract MsgHandlers from update function match arms
pub fn extract_msg_handlers(file_path: &str, syntax_tree: &File) -> Vec<MsgHandler> {
    let mut visitor = HandlerVisitor {
        file_path: file_path.to_string(),
        handlers: Vec::new(),
        in_update_fn: false,
    };

    visitor.visit_file(syntax_tree);
    visitor.handlers
}

struct HandlerVisitor {
    file_path: String,
    handlers: Vec<MsgHandler>,
    in_update_fn: bool,
}

impl<'ast> Visit<'ast> for HandlerVisitor {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        // Check if this is the update function
        if fn_name == "update" {
            self.in_update_fn = true;
            syn::visit::visit_impl_item_fn(self, node);
            self.in_update_fn = false;
        } else {
            syn::visit::visit_impl_item_fn(self, node);
        }
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        if !self.in_update_fn {
            syn::visit::visit_expr_match(self, node);
            return;
        }

        // Check if matching on msg
        let matched_expr = expr_to_string(&node.expr);
        if matched_expr == "msg" || matched_expr.ends_with("msg") {
            // Extract handlers from each arm
            for arm in &node.arms {
                if let Some(handler) = extract_handler_from_arm(arm, &self.file_path) {
                    self.handlers.push(handler);
                }
            }
        }

        syn::visit::visit_expr_match(self, node);
    }
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => {
            path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")
        }
        _ => "<expr>".to_string(),
    }
}

fn extract_handler_from_arm(arm: &Arm, file_path: &str) -> Option<MsgHandler> {
    let msg_pattern = pattern_to_string(&arm.pat);

    // Extract state mutations from the arm body
    let mutations = extract_mutations_from_expr(&arm.body, file_path);

    if mutations.is_empty() {
        return None;
    }

    Some(MsgHandler {
        msg_pattern,
        state_mutations: mutations,
        file_path: file_path.to_string(),
    })
}

fn pattern_to_string(pat: &Pat) -> String {
    match pat {
        Pat::Path(path) => {
            path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")
        }
        Pat::TupleStruct(ts) => {
            ts.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")
        }
        Pat::Struct(s) => {
            s.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")
        }
        Pat::Ident(ident) => ident.ident.to_string(),
        _ => "<pattern>".to_string(),
    }
}

fn extract_mutations_from_expr(expr: &Expr, file_path: &str) -> Vec<StateMutation> {
    let mut mutations = Vec::new();
    let mut visitor = MutationExprVisitor {
        file_path: file_path.to_string(),
        mutations: &mut mutations,
    };
    visitor.visit_expr(expr);
    mutations
}

struct MutationExprVisitor<'a> {
    file_path: String,
    mutations: &'a mut Vec<StateMutation>,
}

impl<'ast, 'a> Visit<'ast> for MutationExprVisitor<'a> {
    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        let target = expr_to_target(&node.left);
        if is_model_mutation(&target) {
            self.mutations.push(StateMutation {
                target,
                mutation_type: "assign".to_string(),
                context: "update".to_string(),
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
            let target = expr_to_target(&node.left);
            if is_model_mutation(&target) {
                self.mutations.push(StateMutation {
                    target,
                    mutation_type: mutation_type.to_string(),
                    context: "update".to_string(),
                    file_path: self.file_path.clone(),
                    line: 0,
                });
            }
        }

        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();
        let mutating_methods = ["push", "pop", "insert", "remove", "clear", "toggle"];

        if mutating_methods.contains(&method.as_str()) {
            let receiver = expr_to_target(&node.receiver);
            if is_model_mutation(&receiver) {
                self.mutations.push(StateMutation {
                    target: receiver,
                    mutation_type: format!("method:{}", method),
                    context: "update".to_string(),
                    file_path: self.file_path.clone(),
                    line: 0,
                });
            }
        }

        syn::visit::visit_expr_method_call(self, node);
    }
}

fn expr_to_target(expr: &Expr) -> String {
    match expr {
        Expr::Path(path) => {
            path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")
        }
        Expr::Field(field) => {
            let base = expr_to_target(&field.base);
            match &field.member {
                syn::Member::Named(ident) => format!("{}.{}", base, ident),
                syn::Member::Unnamed(idx) => format!("{}.{}", base, idx.index),
            }
        }
        _ => "<expr>".to_string(),
    }
}

fn is_model_mutation(target: &str) -> bool {
    target.starts_with("model.") || target.starts_with("state.") || target.starts_with("self.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_button_on_click() {
        let code = r#"
            fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
                Button::primary("+").on_click(ctx, Msg::Increment);
                Button::secondary("-").on_click(ctx, Msg::Decrement);
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let emissions = extract_msg_emissions("test.rs", &syntax_tree);

        assert_eq!(emissions.len(), 2);
        assert_eq!(emissions[0].component, "Button");
        assert_eq!(emissions[0].variant, "primary");
        assert_eq!(emissions[0].label, Some("+".to_string()));
        assert_eq!(emissions[0].msg, "Msg::Increment");

        assert_eq!(emissions[1].component, "Button");
        assert_eq!(emissions[1].variant, "secondary");
        assert_eq!(emissions[1].label, Some("-".to_string()));
        assert_eq!(emissions[1].msg, "Msg::Decrement");
    }

    #[test]
    fn test_extract_msg_handlers() {
        let code = r#"
            impl App for MyApp {
                fn update(model: &mut Model, msg: Msg) -> Cmd<Msg> {
                    match msg {
                        Msg::Increment => model.counter += 1,
                        Msg::Decrement => model.counter -= 1,
                        Msg::Reset => model.counter = 0,
                    }
                    Cmd::none()
                }
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let handlers = extract_msg_handlers("test.rs", &syntax_tree);

        assert_eq!(handlers.len(), 3);
        assert_eq!(handlers[0].msg_pattern, "Msg::Increment");
        assert_eq!(handlers[0].state_mutations[0].target, "model.counter");
        assert_eq!(handlers[0].state_mutations[0].mutation_type, "add_assign");
    }
}
