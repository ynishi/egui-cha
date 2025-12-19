//! Procedural macros for egui-cha layout DSL
//!
//! Provides a declarative syntax for composing layouts that expands to
//! the Fluent Builder API.
//!
//! # Example
//!
//! ```ignore
//! use egui_cha_macros::cha;
//!
//! cha! {
//!     Col(spacing: 8.0) {
//!         Row {
//!             [|ui| ui.label("Hello")]
//!             Spacer
//!         }
//!         Grid(3, gap: 4.0) {
//!             [|ui| ui.label("A")]
//!             [|ui| ui.label("B")]
//!             [|ui| ui.label("C")]
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    Expr, Ident, Result, Token,
};

/// Main macro for declarative layout composition
///
/// Syntax:
/// ```ignore
/// cha! {
///     Col(spacing: 8.0, padding: 4.0) {
///         Row(fill_x) {
///             [closure or expression]
///             Spacer
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn cha(input: TokenStream) -> TokenStream {
    let layout = parse_macro_input!(input as LayoutNode);
    let expanded = layout.to_tokens();
    TokenStream::from(expanded)
}

// ============================================================
// AST Nodes
// ============================================================

/// Root layout node
enum LayoutNode {
    Col(LayoutContainer),
    Row(LayoutContainer),
    Grid(GridContainer),
    Spacer,
    Space(Expr),
    Closure(Expr),
    Expr(Expr),
}

/// Container for Col/Row with optional properties and children
struct LayoutContainer {
    props: Vec<LayoutProp>,
    children: Vec<LayoutNode>,
}

/// Container for Grid with column count, properties, and children
struct GridContainer {
    cols: Expr,
    props: Vec<LayoutProp>,
    children: Vec<LayoutNode>,
}

/// Property like `spacing: 8.0` or `fill_x`
enum LayoutProp {
    KeyValue { key: Ident, value: Expr },
    Flag(Ident),
}

// ============================================================
// Parsing
// ============================================================

impl Parse for LayoutNode {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check for bracketed closure/expression: [expr]
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let expr: Expr = content.parse()?;
            return Ok(LayoutNode::Closure(expr));
        }

        // Check for identifier (Col, Row, Grid, Spacer, Space)
        let ident: Ident = input.parse()?;
        let name = ident.to_string();

        match name.as_str() {
            "Col" => {
                let container = parse_layout_container(input)?;
                Ok(LayoutNode::Col(container))
            }
            "Row" => {
                let container = parse_layout_container(input)?;
                Ok(LayoutNode::Row(container))
            }
            "Grid" => {
                let grid = parse_grid_container(input)?;
                Ok(LayoutNode::Grid(grid))
            }
            "Spacer" => Ok(LayoutNode::Spacer),
            "Space" => {
                let content;
                parenthesized!(content in input);
                let size: Expr = content.parse()?;
                Ok(LayoutNode::Space(size))
            }
            _ => {
                // Treat as expression identifier
                Ok(LayoutNode::Expr(syn::parse_quote!(#ident)))
            }
        }
    }
}

fn parse_layout_container(input: ParseStream) -> Result<LayoutContainer> {
    let mut props = Vec::new();
    let mut children = Vec::new();

    // Parse optional properties in parentheses: (spacing: 8.0, fill_x)
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        props = parse_props(&content)?;
    }

    // Parse children in braces: { ... }
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        children = parse_children(&content)?;
    }

    Ok(LayoutContainer { props, children })
}

fn parse_grid_container(input: ParseStream) -> Result<GridContainer> {
    let content;
    parenthesized!(content in input);

    // First argument is column count
    let cols: Expr = content.parse()?;

    let mut props = Vec::new();

    // Parse remaining properties after comma
    if content.peek(Token![,]) {
        content.parse::<Token![,]>()?;
        props = parse_props(&content)?;
    }

    // Parse children
    let mut children = Vec::new();
    if input.peek(syn::token::Brace) {
        let brace_content;
        braced!(brace_content in input);
        children = parse_children(&brace_content)?;
    }

    Ok(GridContainer {
        cols,
        props,
        children,
    })
}

fn parse_props(input: ParseStream) -> Result<Vec<LayoutProp>> {
    let mut props = Vec::new();

    while !input.is_empty() {
        let ident: Ident = input.parse()?;

        if input.peek(Token![:]) {
            // Key-value property: key: value
            input.parse::<Token![:]>()?;
            let value: Expr = input.parse()?;
            props.push(LayoutProp::KeyValue { key: ident, value });
        } else {
            // Flag property: fill_x, centered
            props.push(LayoutProp::Flag(ident));
        }

        // Optional comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }

    Ok(props)
}

fn parse_children(input: ParseStream) -> Result<Vec<LayoutNode>> {
    let mut children = Vec::new();

    while !input.is_empty() {
        let child: LayoutNode = input.parse()?;
        children.push(child);
    }

    Ok(children)
}

// ============================================================
// Code Generation
// ============================================================

impl LayoutNode {
    fn to_tokens(&self) -> TokenStream2 {
        match self {
            LayoutNode::Col(container) => {
                let props = container.props.iter().map(prop_to_tokens);
                let children = container.children.iter().map(|c| {
                    let child_tokens = c.to_tokens();
                    quote! { .add(#child_tokens) }
                });

                quote! {
                    ::egui_cha_ds::cha::col()
                        #(#props)*
                        #(#children)*
                }
            }
            LayoutNode::Row(container) => {
                let props = container.props.iter().map(prop_to_tokens);
                let children = container.children.iter().map(|c| {
                    let child_tokens = c.to_tokens();
                    quote! { .add(#child_tokens) }
                });

                quote! {
                    ::egui_cha_ds::cha::row()
                        #(#props)*
                        #(#children)*
                }
            }
            LayoutNode::Grid(grid) => {
                let cols = &grid.cols;
                let props = grid.props.iter().map(prop_to_tokens);
                let children = grid.children.iter().map(|c| {
                    let child_tokens = c.to_tokens();
                    quote! { .add(#child_tokens) }
                });

                quote! {
                    ::egui_cha_ds::cha::grid(#cols)
                        #(#props)*
                        #(#children)*
                }
            }
            LayoutNode::Spacer => {
                quote! { ::egui_cha_ds::cha::spacer() }
            }
            LayoutNode::Space(size) => {
                quote! { ::egui_cha_ds::cha::space(#size) }
            }
            LayoutNode::Closure(expr) => {
                quote! { #expr }
            }
            LayoutNode::Expr(expr) => {
                quote! { #expr }
            }
        }
    }
}

fn prop_to_tokens(prop: &LayoutProp) -> TokenStream2 {
    match prop {
        LayoutProp::KeyValue { key, value } => {
            quote! { .#key(#value) }
        }
        LayoutProp::Flag(flag) => {
            quote! { .#flag() }
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_col() {
        let input: TokenStream2 = quote! {
            Col {
                [|ui: &mut egui::Ui| { ui.label("Hello"); }]
            }
        };
        let node: LayoutNode = syn::parse2(input).unwrap();
        let _output = node.to_tokens();
    }

    #[test]
    fn test_parse_col_with_props() {
        let input: TokenStream2 = quote! {
            Col(spacing: 8.0, fill_x) {
                [|ui: &mut egui::Ui| { ui.label("Test"); }]
            }
        };
        let node: LayoutNode = syn::parse2(input).unwrap();
        let _output = node.to_tokens();
    }

    #[test]
    fn test_parse_nested() {
        let input: TokenStream2 = quote! {
            Col {
                Row {
                    [|ui: &mut egui::Ui| { ui.label("A"); }]
                    Spacer
                }
            }
        };
        let node: LayoutNode = syn::parse2(input).unwrap();
        let _output = node.to_tokens();
    }

    #[test]
    fn test_parse_grid() {
        let input: TokenStream2 = quote! {
            Grid(3, gap: 4.0) {
                [|ui: &mut egui::Ui| { ui.label("1"); }]
                [|ui: &mut egui::Ui| { ui.label("2"); }]
                [|ui: &mut egui::Ui| { ui.label("3"); }]
            }
        };
        let node: LayoutNode = syn::parse2(input).unwrap();
        let _output = node.to_tokens();
    }
}
