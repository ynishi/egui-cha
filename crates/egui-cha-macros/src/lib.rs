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
    Scroll(ScrollContainer),
    Card(CardContainer),
    Icon(IconNode),
    Expr(Expr),
}

/// Scroll container with direction and optional properties
struct ScrollContainer {
    direction: ScrollDirection,
    props: Vec<LayoutProp>,
    children: Vec<LayoutNode>,
}

/// Scroll direction for Scroll node
#[derive(Clone, Copy)]
enum ScrollDirection {
    Vertical,
    Horizontal,
    Both,
}

/// Card container with optional title
struct CardContainer {
    title: Option<Expr>,
    props: Vec<LayoutProp>,
    children: Vec<LayoutNode>,
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
                "Scroll" | "ScrollH" | "ScrollBoth" => {
                    let _: Ident = input.parse()?;
                    let direction = match name.as_str() {
                        "Scroll" => ScrollDirection::Vertical,
                        "ScrollH" => ScrollDirection::Horizontal,
                        "ScrollBoth" => ScrollDirection::Both,
                        _ => unreachable!(),
                    };
                    let container = parse_container(input)?;
                    return Ok(LayoutNode::Scroll(ScrollContainer {
                        direction,
                        props: container.props,
                        children: container.children,
                    }));
                }
                "Card" => {
                    let _: Ident = input.parse()?;
                    let (title, props, children) = parse_card(input)?;
                    return Ok(LayoutNode::Card(CardContainer {
                        title,
                        props,
                        children,
                    }));
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

/// Parse Card: Card("title") { ... } or Card { ... }
fn parse_card(input: ParseStream) -> Result<(Option<Expr>, Vec<LayoutProp>, Vec<LayoutNode>)> {
    let mut title = None;
    let mut props = Vec::new();
    let mut children = Vec::new();

    // Parse optional title and props: ("title") or ("title", padding: 8.0)
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);

        // First item could be title (string literal) or property
        if !content.is_empty() {
            let fork = content.fork();
            // Check if it starts with an identifier followed by `:` (property)
            if fork.peek(Ident) {
                let _: Ident = fork.parse()?;
                if fork.peek(Token![:]) {
                    // It's properties, no title
                    props = parse_props(&content)?;
                } else {
                    // It's something else, try as title expression
                    title = Some(content.parse::<Expr>()?);
                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                        props = parse_props(&content)?;
                    }
                }
            } else {
                // Not an identifier, must be title (e.g., string literal)
                title = Some(content.parse::<Expr>()?);
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                    props = parse_props(&content)?;
                }
            }
        }
    }

    // Parse children: { ... }
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        while !content.is_empty() {
            children.push(LayoutNode::parse(&content)?);
        }
    }

    Ok((title, props, children))
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
            LayoutNode::Scroll(scroll) => {
                let scroll_props = scroll_props_to_builder(&scroll.props);
                let children = scroll.children.iter().map(|c| c.to_tokens(ctx));
                let constructor = match scroll.direction {
                    ScrollDirection::Vertical => quote! { ::egui_cha::ScrollArea::vertical() },
                    ScrollDirection::Horizontal => quote! { ::egui_cha::ScrollArea::horizontal() },
                    ScrollDirection::Both => quote! { ::egui_cha::ScrollArea::both() },
                };

                quote! {
                    #constructor #scroll_props .show_ctx(#ctx, |#ctx| {
                        #(#children)*
                    });
                }
            }
            LayoutNode::Card(card) => {
                let card_props = card_props_to_builder(&card.props);
                let children = card.children.iter().map(|c| c.to_tokens(ctx));
                let constructor = if let Some(title) = &card.title {
                    quote! { ::egui_cha_ds::Card::titled(#title) }
                } else {
                    quote! { ::egui_cha_ds::Card::new() }
                };

                quote! {
                    #constructor #card_props .show_ctx(#ctx, |#ctx| {
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

/// Convert ScrollArea properties to builder method calls
fn scroll_props_to_builder(props: &[LayoutProp]) -> TokenStream2 {
    let methods: Vec<TokenStream2> = props
        .iter()
        .filter_map(|prop| {
            let key = prop.key.to_string();
            let value = &prop.value;

            match key.as_str() {
                "max_height" => Some(quote! { .max_height(#value) }),
                "max_width" => Some(quote! { .max_width(#value) }),
                "min_height" => Some(quote! { .min_scrolled_height(#value) }),
                "min_width" => Some(quote! { .min_scrolled_width(#value) }),
                "id" => Some(quote! { .id_salt(#value) }),
                _ => None,
            }
        })
        .collect();

    quote! { #(#methods)* }
}

/// Convert Card properties to builder method calls
fn card_props_to_builder(props: &[LayoutProp]) -> TokenStream2 {
    let methods: Vec<TokenStream2> = props
        .iter()
        .filter_map(|prop| {
            let key = prop.key.to_string();
            let value = &prop.value;

            match key.as_str() {
                "padding" => Some(quote! { .padding(#value) }),
                _ => None,
            }
        })
        .collect();

    quote! { #(#methods)* }
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

    #[test]
    fn test_parse_scroll() {
        let input: TokenStream2 = quote! {
            ctx, {
                Scroll(max_height: 300.0) {
                    ctx.ui.label("Scrollable")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("ScrollArea"));
        assert!(code.contains("vertical"));
        assert!(code.contains("max_height"));
    }

    #[test]
    fn test_parse_scroll_horizontal() {
        let input: TokenStream2 = quote! {
            ctx, {
                ScrollH {
                    ctx.ui.label("Horizontal")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("ScrollArea"));
        assert!(code.contains("horizontal"));
    }

    #[test]
    fn test_parse_scroll_both() {
        let input: TokenStream2 = quote! {
            ctx, {
                ScrollBoth(max_height: 400.0, max_width: 600.0) {
                    ctx.ui.label("Both directions")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("ScrollArea"));
        assert!(code.contains("both"));
        assert!(code.contains("max_height"));
        assert!(code.contains("max_width"));
    }

    #[test]
    fn test_parse_card_with_title() {
        let input: TokenStream2 = quote! {
            ctx, {
                Card("My Card") {
                    ctx.ui.label("Content")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("Card"));
        assert!(code.contains("titled"));
    }

    #[test]
    fn test_parse_card_without_title() {
        let input: TokenStream2 = quote! {
            ctx, {
                Card {
                    ctx.ui.label("Content")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("Card"));
        assert!(code.contains("new"));
    }

    #[test]
    fn test_parse_card_with_props() {
        let input: TokenStream2 = quote! {
            ctx, {
                Card("Title", padding: 8.0) {
                    ctx.ui.label("Content")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("Card"));
        assert!(code.contains("titled"));
        assert!(code.contains("padding"));
    }
}
