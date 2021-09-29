//! This module generates AST datatype used by rust-analyzer.
//!
//! Specifically, it generates the `SyntaxKind` enum and a number of newtype
//! wrappers around `SyntaxNode` which implement `syntax::AstNode`.

mod decl;

use quote::quote;
use ungrammar::Grammar;

use crate::syntax::decl::token_name;

pub struct GeneratedSyntax {
    pub ast: String,
    pub node_kinds: Vec<String>,
    pub token_macro: String,
}

pub fn generate_syntax(ungram: &str) -> Result<GeneratedSyntax, ungrammar::Error> {
    let grammar: Grammar = ungram.parse()?;

    Ok(GeneratedSyntax {
        ast: Default::default(),
        node_kinds: node_kinds(&grammar),
        token_macro: generate_token_macro(&grammar),
    })
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

    quote! {
        #[macro_export]
        macro_rules! T {
            #(#arms)*
        }
        pub use T;
    }
    .to_string()
}

fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_uppercase());
    }
    buf
}
