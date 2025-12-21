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

/// Layout node - container, icon shorthand, control flow, or expression
enum LayoutNode {
    Col(LayoutContainer),
    Row(LayoutContainer),
    Group(LayoutContainer),
    Scroll(ScrollContainer),
    Card(CardContainer),
    Icon(IconNode),
    // Control flow
    If(IfNode),
    IfElse(IfElseNode),
    For(ForNode),
    Enabled(ConditionContainer),
    Visible(ConditionContainer),
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

/// If node: If(condition) { ... }
struct IfNode {
    condition: Expr,
    children: Vec<LayoutNode>,
}

/// IfElse node: IfElse(condition) { ... } Else { ... }
struct IfElseNode {
    condition: Expr,
    if_children: Vec<LayoutNode>,
    else_children: Vec<LayoutNode>,
}

/// For node: For(item in iter) { ... }
struct ForNode {
    pattern: syn::Pat,
    iter: Expr,
    children: Vec<LayoutNode>,
}

/// Condition container for Enabled/Visible: Enabled(condition) { ... }
struct ConditionContainer {
    condition: Expr,
    children: Vec<LayoutNode>,
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
                // Control flow: If(condition) { ... }
                "If" => {
                    let _: Ident = input.parse()?;
                    let (condition, children) = parse_condition_block(input)?;
                    return Ok(LayoutNode::If(IfNode {
                        condition,
                        children,
                    }));
                }
                // Control flow: IfElse(condition) { ... } Else { ... }
                "IfElse" => {
                    let _: Ident = input.parse()?;
                    let (condition, if_children, else_children) = parse_if_else(input)?;
                    return Ok(LayoutNode::IfElse(IfElseNode {
                        condition,
                        if_children,
                        else_children,
                    }));
                }
                // Control flow: For(pattern in iter) { ... }
                "For" => {
                    let _: Ident = input.parse()?;
                    let (pattern, iter, children) = parse_for(input)?;
                    return Ok(LayoutNode::For(ForNode {
                        pattern,
                        iter,
                        children,
                    }));
                }
                // Control flow: Enabled(condition) { ... }
                "Enabled" => {
                    let _: Ident = input.parse()?;
                    let (condition, children) = parse_condition_block(input)?;
                    return Ok(LayoutNode::Enabled(ConditionContainer {
                        condition,
                        children,
                    }));
                }
                // Control flow: Visible(condition) { ... }
                "Visible" => {
                    let _: Ident = input.parse()?;
                    let (condition, children) = parse_condition_block(input)?;
                    return Ok(LayoutNode::Visible(ConditionContainer {
                        condition,
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

/// Parse condition block: (condition) { ... }
fn parse_condition_block(input: ParseStream) -> Result<(Expr, Vec<LayoutNode>)> {
    // Parse condition in parentheses
    let content;
    parenthesized!(content in input);
    let condition: Expr = content.parse()?;

    // Parse children in braces
    let mut children = Vec::new();
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        while !content.is_empty() {
            children.push(LayoutNode::parse(&content)?);
        }
    }

    Ok((condition, children))
}

/// Parse IfElse: (condition) { ... } Else { ... }
fn parse_if_else(input: ParseStream) -> Result<(Expr, Vec<LayoutNode>, Vec<LayoutNode>)> {
    // Parse condition in parentheses
    let content;
    parenthesized!(content in input);
    let condition: Expr = content.parse()?;

    // Parse if-children in braces
    let mut if_children = Vec::new();
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        while !content.is_empty() {
            if_children.push(LayoutNode::parse(&content)?);
        }
    }

    // Parse Else keyword and else-children
    let mut else_children = Vec::new();
    if input.peek(Ident) {
        let fork = input.fork();
        let ident: Ident = fork.parse()?;
        if ident == "Else" {
            let _: Ident = input.parse()?; // consume "Else"
            if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                while !content.is_empty() {
                    else_children.push(LayoutNode::parse(&content)?);
                }
            }
        }
    }

    Ok((condition, if_children, else_children))
}

/// Parse For: (pattern in iter) { ... }
fn parse_for(input: ParseStream) -> Result<(syn::Pat, Expr, Vec<LayoutNode>)> {
    // Parse (pattern in iter)
    let content;
    parenthesized!(content in input);

    // Parse pattern (e.g., `item`, `(key, value)`)
    let pattern = syn::Pat::parse_single(&content)?;

    // Parse `in` keyword
    content.parse::<Token![in]>()?;

    // Parse iterator expression
    let iter: Expr = content.parse()?;

    // Parse children in braces
    let mut children = Vec::new();
    if input.peek(syn::token::Brace) {
        let content;
        braced!(content in input);
        while !content.is_empty() {
            children.push(LayoutNode::parse(&content)?);
        }
    }

    Ok((pattern, iter, children))
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
            // Control flow: If(condition) { ... } -> ctx.show_if(condition, |ctx| { ... })
            LayoutNode::If(if_node) => {
                let condition = &if_node.condition;
                let children = if_node.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.show_if(#condition, |#ctx| {
                        #(#children)*
                    });
                }
            }
            // Control flow: IfElse(condition) { ... } Else { ... }
            LayoutNode::IfElse(if_else) => {
                let condition = &if_else.condition;
                let if_children = if_else.if_children.iter().map(|c| c.to_tokens(ctx));
                let else_children = if_else.else_children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.show_if_else(
                        #condition,
                        |#ctx| { #(#if_children)* },
                        |#ctx| { #(#else_children)* }
                    );
                }
            }
            // Control flow: For(pattern in iter) { ... } -> for pattern in iter { ... }
            LayoutNode::For(for_node) => {
                let pattern = &for_node.pattern;
                let iter = &for_node.iter;
                let children = for_node.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    for #pattern in #iter {
                        #(#children)*
                    }
                }
            }
            // Control flow: Enabled(condition) { ... } -> ctx.enabled_if(condition, |ctx| { ... })
            LayoutNode::Enabled(container) => {
                let condition = &container.condition;
                let children = container.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.enabled_if(#condition, |#ctx| {
                        #(#children)*
                    });
                }
            }
            // Control flow: Visible(condition) { ... } -> ctx.visible_if(condition, |ctx| { ... })
            LayoutNode::Visible(container) => {
                let condition = &container.condition;
                let children = container.children.iter().map(|c| c.to_tokens(ctx));

                quote! {
                    #ctx.visible_if(#condition, |#ctx| {
                        #(#children)*
                    });
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

    #[test]
    fn test_parse_if() {
        let input: TokenStream2 = quote! {
            ctx, {
                If(model.is_admin) {
                    ctx.ui.label("Admin")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("show_if"));
        assert!(code.contains("is_admin"));
    }

    #[test]
    fn test_parse_if_else() {
        let input: TokenStream2 = quote! {
            ctx, {
                IfElse(model.loading) {
                    ctx.ui.spinner()
                } Else {
                    ctx.ui.label("Loaded")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("show_if_else"));
        assert!(code.contains("loading"));
    }

    #[test]
    fn test_parse_for() {
        let input: TokenStream2 = quote! {
            ctx, {
                For(item in &model.items) {
                    ctx.ui.label(&item.name)
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("for"));
        assert!(code.contains("item"));
        assert!(code.contains("items"));
    }

    #[test]
    fn test_parse_enabled() {
        let input: TokenStream2 = quote! {
            ctx, {
                Enabled(model.can_submit) {
                    ctx.ui.button("Submit")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("enabled_if"));
        assert!(code.contains("can_submit"));
    }

    #[test]
    fn test_parse_visible() {
        let input: TokenStream2 = quote! {
            ctx, {
                Visible(model.show_details) {
                    ctx.ui.label("Details")
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("visible_if"));
        assert!(code.contains("show_details"));
    }

    #[test]
    fn test_parse_nested_control_flow() {
        let input: TokenStream2 = quote! {
            ctx, {
                Col {
                    If(model.show_list) {
                        For(item in &model.items) {
                            Card(&item.title) {
                                ctx.ui.label(&item.content)
                            }
                        }
                    }
                }
            }
        };
        let parsed: ChaInput = syn::parse2(input).unwrap();
        let tokens = parsed.to_tokens();
        let code = tokens.to_string();
        assert!(code.contains("vertical"));
        assert!(code.contains("show_if"));
        assert!(code.contains("for"));
        assert!(code.contains("Card"));
    }
}
