//! This module generates AST datatype used by rust-analyzer.
//!
//! Specifically, it generates the `SyntaxKind` enum and a number of new type
//! wrappers around `SyntaxNode` which implement `syntax::AstNode`.

mod decl;

use std::collections::HashMap;

use quote::{format_ident, quote};
use ungrammar::{Grammar, Node, Rule, Token};

use crate::syntax::decl::token_name;

pub struct GeneratedSyntax {
    pub ast: String,
    pub node_kinds: Vec<String>,
    pub token_macro: String,
}

pub fn generate_syntax(ungram: &str) -> Result<GeneratedSyntax, ungrammar::Error> {
    let grammar: Grammar = ungram.parse()?;

    Ok(GeneratedSyntax {
        ast: generate_ast(&grammar),
        node_kinds: node_kinds(&grammar),
        token_macro: generate_token_macro(&grammar),
    })
}

fn generate_ast(grammar: &Grammar) -> String {
    let mut ast_code = quote! {
        use crate::syntax::{SyntaxNode, SyntaxToken, SyntaxKind::*};

        pub trait AstNode: Sized {
            fn can_cast(syntax: &SyntaxNode) -> bool;
            fn cast(syntax: SyntaxNode) -> Option<Self>;
            fn syntax(&self) -> SyntaxNode;
        }
    };

    for idx in grammar.iter() {
        let node = &grammar[idx];
        let node_ident = format_ident!("{}", &node.name);
        let node_kind = format_ident!("{}", &to_upper_snake_case(&node.name));

        match &node.rule {
            ungrammar::Rule::Node(n) => {
                let inner_node = &grammar[*n];
                let inner_node_ident = format_ident!("{}", &inner_node.name);
                let inner_get_name = format_ident!("{}", &node_getter_name(&inner_node.name));

                ast_code.extend(quote! {
                    #[derive(Debug, Clone)]
                    pub struct #node_ident(SyntaxNode);

                    impl AstNode for #node_ident {
                        #[inline]
                        fn can_cast(syntax: &SyntaxNode) -> bool {
                            syntax.kind() == #node_kind
                        }

                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(&syntax) {
                                Some(Self(syntax))
                            } else {
                                None
                            }
                        }

                        fn syntax(&self) -> SyntaxNode {
                            self.0.clone()
                        }
                    }

                    impl #node_ident {
                        pub fn #inner_get_name(&self) -> Option<#inner_node_ident> {
                            self
                                .0
                                .first_child()
                                .and_then(#inner_node_ident::cast)
                        }
                    }
                });
            }
            ungrammar::Rule::Token(t) => {
                let inner_token = &grammar[*t];

                let inner_get_name = format_ident!("{}", &token_getter_name(&inner_token.name));

                ast_code.extend(quote! {
                    #[derive(Debug, Clone)]
                    pub struct #node_ident(SyntaxNode);

                    impl AstNode for #node_ident {
                        #[inline]
                        fn can_cast(syntax: &SyntaxNode) -> bool {
                            syntax.kind() == #node_kind
                        }

                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(&syntax) {
                                Some(Self(syntax))
                            } else {
                                None
                            }
                        }

                        fn syntax(&self) -> SyntaxNode {
                            self.0.clone()
                        }
                    }

                    impl #node_ident {
                        pub fn #inner_get_name(&self) -> Option<SyntaxToken> {
                            self
                                .0
                                .first_child_or_token()
                                .and_then(|e| e.into_token())
                        }
                    }
                });
            }
            ungrammar::Rule::Alt(rules) => {
                if rules.iter().all(|r| matches!(r, &Rule::Token(_))) {
                    ast_code.extend(quote! {
                        #[derive(Debug, Clone)]
                        pub struct #node_ident(SyntaxNode);

                        impl AstNode for #node_ident {
                            #[inline]
                            fn can_cast(syntax: &SyntaxNode) -> bool {
                                syntax.kind() == #node_kind
                            }

                            #[inline]
                            fn cast(syntax: SyntaxNode) -> Option<Self> {
                                if !Self::can_cast(&syntax) {
                                    return None;
                                }

                                Some(Self(syntax))
                            }

                            fn syntax(&self) -> SyntaxNode {
                                self.0.clone()
                            }
                        }

                        impl #node_ident {
                            pub fn token(&self) -> Option<SyntaxToken> {
                                self
                                    .0
                                    .first_child_or_token()
                                    .and_then(|e| e.into_token())
                            }
                        }
                    });
                    continue;
                }

                if rules.iter().any(|r| !matches!(r, &Rule::Node(_))) {
                    ast_code.extend(quote! {
                        #[derive(Debug, Clone)]
                        pub struct #node_ident(SyntaxNode);

                        impl AstNode for #node_ident {
                            #[inline]
                            fn can_cast(syntax: &SyntaxNode) -> bool {
                                syntax.kind() == #node_kind
                            }

                            #[inline]
                            fn cast(syntax: SyntaxNode) -> Option<Self> {
                                if Self::can_cast(&syntax) {
                                    Some(Self(syntax))
                                } else {
                                    None
                                }
                            }

                            fn syntax(&self) -> SyntaxNode {
                                self.0.clone()
                            }
                        }
                    });

                    continue;
                }

                let mut variants = quote! {};

                let mut syntax_match_arms = quote! {};
                let mut cast_match_arms = quote! {};

                for rule in rules {
                    match rule {
                        Rule::Node(n) => {
                            let inner_node = &grammar[*n];
                            let inner_node_ident = format_ident!("{}", &inner_node.name);
                            let inner_node_kind_ident =
                                format_ident!("{}", to_upper_snake_case(&inner_node.name));
                            let inner_variant_ident = format_ident!(
                                "{}",
                                &strip_common_prefix(&inner_node.name, &node.name)
                            );

                            variants.extend(quote! {
                                #inner_variant_ident(#inner_node_ident),
                            });

                            syntax_match_arms.extend(quote! {
                                Self::#inner_variant_ident(t) => t.syntax().parent().unwrap(),
                            });

                            cast_match_arms.extend(quote! {
                                #inner_node_kind_ident => Some(Self::#inner_variant_ident(#inner_node_ident::cast(first_child)?)),
                            });
                        }
                        _ => unreachable!(),
                    }
                }

                ast_code.extend(quote! {
                    #[derive(Debug, Clone)]
                    pub enum #node_ident {
                        #variants
                    }

                    impl AstNode for #node_ident {
                        #[inline]
                        fn can_cast(syntax: &SyntaxNode) -> bool {
                            syntax.kind() == #node_kind
                        }

                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if !Self::can_cast(&syntax) {
                                return None;
                            }

                            let first_child = syntax.first_child()?;

                            match first_child.kind() {
                                #cast_match_arms
                                _ => None
                            }
                        }

                        fn syntax(&self) -> SyntaxNode {
                            match self {
                                #syntax_match_arms
                            }
                        }
                    }
                });
            }
            ungrammar::Rule::Seq(rules) => {
                let mut getters = quote! {};

                // For identical kinds.
                let mut node_skip_count = HashMap::<Node, usize>::new();
                let mut token_skip_count = HashMap::<Token, usize>::new();

                for inner_rule in rules {
                    let label = match inner_rule {
                        Rule::Labeled { label, rule: _ } => Some(label.clone()),
                        _ => None,
                    };

                    let rule = match inner_rule {
                        Rule::Labeled { label: _, rule } => &*rule,
                        r => r,
                    };

                    if label.as_ref().map_or(false, |l| l.starts_with("__")) {
                        continue;
                    }

                    match rule {
                        Rule::Node(n) => {
                            let inner_node = &grammar[*n];
                            let inner_node_ident = format_ident!("{}", &inner_node.name);

                            let getter_name =
                                label.unwrap_or_else(|| to_lower_snake_case(&inner_node.name));
                            let getter_ident = format_ident!("{}", &getter_name);

                            let skip_count = node_skip_count.get(n).copied().unwrap_or_default();
                            *node_skip_count.entry(*n).or_insert(0) += 1;

                            getters.extend(quote! {
                                pub fn #getter_ident(&self) -> Option<#inner_node_ident> {
                                    self.0.children().filter_map(#inner_node_ident::cast).skip(#skip_count).next()
                                }
                            });
                        }
                        Rule::Token(t) => {
                            let inner_token = &grammar[*t];
                            let token_kind_ident =
                                format_ident!("{}", token_name(&inner_token.name));

                            let getter_ident = format_ident!(
                                "{}",
                                &label.unwrap_or_else(|| token_getter_name(&inner_token.name))
                            );

                            let skip_count = token_skip_count.get(t).copied().unwrap_or_default();
                            *token_skip_count.entry(*t).or_insert(0) += 1;

                            getters.extend(quote! {
                                pub fn #getter_ident(&self) -> Option<SyntaxToken> {
                                    self.0.children_with_tokens().filter_map(|t| {
                                        if t.kind() != #token_kind_ident {
                                            return None
                                        }
                                        t.into_token()
                                    })
                                    .skip(#skip_count)
                                    .next()
                                }
                            });
                        }
                        Rule::Rep(rule) => match &**rule {
                            Rule::Node(n) => {
                                let repeated_node = &grammar[*n];
                                let repeated_node_ident = format_ident!("{}", &repeated_node.name);

                                let getter_name = label.unwrap_or_else(|| {
                                    pluralize(&to_lower_snake_case(&repeated_node.name))
                                });
                                let getter_ident = format_ident!("{}", &getter_name);

                                getters.extend(quote! {
                                        pub fn #getter_ident(&self) -> impl Iterator<Item = #repeated_node_ident> {
                                            self.0.children().filter_map(#repeated_node_ident::cast)
                                        }
                                    });
                            }
                            _ => {}
                        },
                        Rule::Seq(_) | Rule::Alt(_) | Rule::Opt(_) => {}
                        Rule::Labeled { .. } => unreachable!(),
                    }
                }

                ast_code.extend(quote! {
                    #[derive(Debug, Clone)]
                    pub struct #node_ident(SyntaxNode);

                    impl AstNode for #node_ident {
                        #[inline]
                        fn can_cast(syntax: &SyntaxNode) -> bool {
                            syntax.kind() == #node_kind
                        }

                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(&syntax) {
                                Some(Self(syntax))
                            } else {
                                None
                            }
                        }

                        fn syntax(&self) -> SyntaxNode {
                            self.0.clone()
                        }
                    }

                    impl #node_ident {
                        #getters
                    }
                });
            }
            _ => {
                ast_code.extend(quote! {
                    #[derive(Debug, Clone)]
                    pub struct #node_ident(SyntaxNode);

                    impl AstNode for #node_ident {
                        #[inline]
                        fn can_cast(syntax: &SyntaxNode) -> bool {
                            syntax.kind() == #node_kind
                        }

                        #[inline]
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(&syntax) {
                                Some(Self(syntax))
                            } else {
                                None
                            }
                        }

                        fn syntax(&self) -> SyntaxNode {
                            self.0.clone()
                        }
                    }
                });
            }
        }
    }

    ast_code.to_string()
}

fn node_kinds(grammar: &Grammar) -> Vec<String> {
    grammar
        .iter()
        .map(|n| to_upper_snake_case(&grammar[n].name))
        .collect()
}

fn generate_token_macro(grammar: &Grammar) -> String {
    let arms = grammar.tokens().map(|t| {
        let t = &grammar[t].name;
        let name = quote::format_ident!("{}", token_name(t));

        quote! {[#t] => { $crate::syntax::SyntaxKind::#name };}
    });

    String::from(r#"/// A macro for using tokens in a more humanly way, e.g. `T!["="]`."#)
        + "\n"
        + &(quote! {
            #[macro_export]
            macro_rules! T {
                #(#arms)*
            }
            pub use T;
        })
        .to_string()
}

fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_');
        }
        prev = true;

        buf.push(c.to_ascii_uppercase());
    }
    buf
}

fn to_lower_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_');
        }
        prev = true;

        buf.push(c.to_ascii_lowercase());
    }
    buf
}

fn node_getter_name(s: &str) -> String {
    to_lower_snake_case(s)
}

fn token_getter_name(s: &str) -> String {
    format!("{}_token", token_name(s).to_lowercase())
}

fn pluralize(s: &str) -> String {
    format!("{}{}", s, if s.ends_with('s') { "es" } else { "s" })
}

fn strip_common_prefix(target: &str, s: &str) -> String {
    let cpl = common_prefix_length(target, s);

    if target[cpl..]
        .chars()
        .next()
        .map_or(true, |c| c.is_ascii_lowercase())
    {
        return target.to_string();
    }

    target[cpl..].to_string()
}

fn common_prefix_length(s1: &str, s2: &str) -> usize {
    s1.bytes()
        .zip(s2.bytes())
        .take_while(|(b1, b2)| b1 == b2)
        .count()
}
