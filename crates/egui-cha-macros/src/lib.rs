//! Procedural macros for egui-cha layout DSL
//!
//! Provides a declarative syntax for composing layouts that integrates
//! directly with ViewCtx.
//!
//! # Example
//!
//! ```ignore
//! use egui_cha_macros::cha;
//!
//! fn view(model: &Model, ctx: &mut ViewCtx<Msg>) {
//!     cha!(ctx, {
//!         Col(spacing: 8.0) {
//!             ctx.ui.heading("Counter")
//!             Row {
//!                 @house           // Icon shorthand
//!                 @gear(20.0)      // Icon with size
//!             }
//!             Row {
//!                 Button::primary("+").on_click(ctx, Msg::Increment)
//!             }
//!         }
//!     });
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, Result, Token,
};

/// Main macro for declarative layout composition with ViewCtx
///
/// Syntax:
/// ```ignore
/// cha!(ctx, {
///     Col(spacing: 8.0) {
///         Row {
///             @house           // Icon::house().show_ctx(ctx)
///             @gear(20.0)      // Icon::gear().size(20.0).show_ctx(ctx)
///         }
///     }
/// });
/// ```
#[proc_macro]
pub fn cha(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ChaInput);
    let expanded = input.to_tokens();
    TokenStream::from(expanded)
}

// ============================================================
// AST Nodes
// ============================================================

/// Top-level input: ctx identifier and body
struct ChaInput {
    ctx: Ident,
    body: LayoutBody,
}

/// Layout body containing children
struct LayoutBody {
    children: Vec<LayoutNode>,
}

/// Layout node - container, icon shorthand, or expression
enum LayoutNode {
    Col(LayoutContainer),
    Row(LayoutContainer),
    Group(LayoutContainer),
    Icon(IconNode),
    Expr(Expr),
}

/// Container with optional properties and children
struct LayoutContainer {
    props: Vec<LayoutProp>,
    children: Vec<LayoutNode>,
}

/// Property like `spacing: 8.0`
struct LayoutProp {
    key: Ident,
    value: Expr,
}

/// Icon shorthand: @house or @gear(20.0)
struct IconNode {
    name: Ident,
    size: Option<Expr>,
}

// ============================================================
// Parsing
// ============================================================

impl Parse for ChaInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse: ctx, { ... }
        let ctx: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        braced!(content in input);
        let body = LayoutBody::parse(&content)?;

        Ok(ChaInput { ctx, body })
    }
}

impl LayoutBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut children = Vec::new();

        while !input.is_empty() {
            let node = LayoutNode::parse(input)?;
            children.push(node);
        }

        Ok(LayoutBody { children })
    }
}

impl LayoutNode {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check for @ (icon shorthand)
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let name: Ident = input.parse()?;

            // Optional size in parentheses
            let size = if input.peek(syn::token::Paren) {
                let content;
                parenthesized!(content in input);
                Some(content.parse::<Expr>()?)
            } else {
                None
            };

            return Ok(LayoutNode::Icon(IconNode { name, size }));
        }

        // Look ahead to determine node type
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            let name = ident.to_string();

            match name.as_str() {
                "Col" | "Row" | "Group" => {
                    // Actually consume the identifier
                    let _: Ident = input.parse()?;
                    let container = parse_container(input)?;

                    return Ok(match name.as_str() {
                        "Col" => LayoutNode::Col(container),
                        "Row" => LayoutNode::Row(container),
                        "Group" => LayoutNode::Group(container),
                        _ => unreachable!(),
                    });
                }
                _ => {}
            }
        }

        // Parse as expression
        let expr: Expr = input.parse()?;
        Ok(LayoutNode::Expr(expr))
    }
}

fn parse_container(input: ParseStream) -> Result<LayoutContainer> {
    let mut props = Vec::new();
    let mut children = Vec::new();

    // Parse optional properties: (spacing: 8.0, padding: 4.0)
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        props = parse_props(&content)?;
    }

    // Parse children: { ... }
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        while !content.is_empty() {
            children.push(LayoutNode::parse(&content)?);
        }
    }

    Ok(LayoutContainer { props, children })
}

fn parse_props(input: ParseStream) -> Result<Vec<LayoutProp>> {
    let mut props = Vec::new();

    while !input.is_empty() {
        let key: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;
        props.push(LayoutProp { key, value });

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    Ok(props)
}

// ============================================================
// Code Generation
// ============================================================

impl ChaInput {
    fn to_tokens(&self) -> TokenStream2 {
        let ctx = &self.ctx;
        let children = self.body.children.iter().map(|c| c.to_tokens(ctx));

        quote! {
            #(#children)*
        }
    }
}

impl LayoutNode {
    fn to_tokens(&self, ctx: &Ident) -> TokenStream2 {
        match self {
            LayoutNode::Col(container) => {
                let setup = props_to_setup(&container.props, "y");
                let children = container.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.vertical(|#ctx| {
                        #setup
                        #(#children)*
                    });
                }
            }
            LayoutNode::Row(container) => {
                let setup = props_to_setup(&container.props, "x");
                let children = container.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.horizontal(|#ctx| {
                        #setup
                        #(#children)*
                    });
                }
            }
            LayoutNode::Group(container) => {
                let setup = props_to_setup(&container.props, "y");
                let children = container.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.group(|#ctx| {
                        #setup
                        #(#children)*
                    });
                }
            }
            LayoutNode::Icon(icon) => {
                let name = &icon.name;
                if let Some(size) = &icon.size {
                    quote! {
                        ::egui_cha_ds::Icon::#name().size(#size).show_ctx(#ctx);
                    }
                } else {
                    quote! {
                        ::egui_cha_ds::Icon::#name().show_ctx(#ctx);
                    }
                }
            }
            LayoutNode::Expr(expr) => {
                quote! { #expr; }
            }
        }
    }
}

fn props_to_setup(props: &[LayoutProp], axis: &str) -> TokenStream2 {
    let setups: Vec<TokenStream2> = props
        .iter()
        .filter_map(|prop| {
            let key = prop.key.to_string();
            let value = &prop.value;

            match key.as_str() {
                "spacing" => {
                    if axis == "y" {
                        Some(quote! { ctx.ui.spacing_mut().item_spacing.y = #value; })
                    } else {
                        Some(quote! { ctx.ui.spacing_mut().item_spacing.x = #value; })
                    }
                }
                "padding" => Some(quote! {
                    ctx.ui.spacing_mut().window_margin = egui::Margin::same(#value);
                }),
                _ => None,
            }
        })
        .collect();

    quote! { #(#setups)* }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let input: TokenStream2 = quote! {
            ctx, {
                ctx.ui.label("Hello")
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let _ = parsed.to_tokens();
    }

    #[test]
    fn test_parse_col() {
        let input: TokenStream2 = quote! {
            ctx, {
                Col {
                    ctx.ui.label("Hello")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let _ = parsed.to_tokens();
    }

    #[test]
    fn test_parse_col_with_props() {
        let input: TokenStream2 = quote! {
            ctx, {
                Col(spacing: 8.0) {
                    ctx.ui.label("Hello")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let _ = parsed.to_tokens();
    }

    #[test]
    fn test_parse_nested() {
        let input: TokenStream2 = quote! {
            ctx, {
                Col(spacing: 8.0) {
                    Row {
                        ctx.ui.label("A")
                        ctx.ui.label("B")
                    }
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let _ = parsed.to_tokens();
    }

    #[test]
    fn test_parse_icon_shorthand() {
        let input: TokenStream2 = quote! {
            ctx, {
                Row {
                    @house
                    @gear(20.0)
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("Icon"));
        assert!(code.contains("house"));
        assert!(code.contains("gear"));
        assert!(code.contains("size"));
    }
}
