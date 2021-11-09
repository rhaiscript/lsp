use rhai_hir::{Module, Symbol, Type};
use rhai_rowan::{
    ast::{AstNode, ExprFn, Rhai},
    syntax::SyntaxElement,
};

pub fn documentation_for(module: &Module, rhai: &Rhai, symbol: Symbol, signature: bool) -> String {
    let sig = if signature {
        signature_of(module, rhai, symbol).wrap_rhai_markdown()
    } else {
        String::new()
    };

    let sym_data = &module[symbol];

    if let Some(docs) = sym_data.docs() {
        return format!(
            "{sig}{docs}",
            sig = sig,
            docs = if docs.is_empty() {
                String::new()
            } else {
                format!("\n{}", docs)
            }
        );
    }

    String::new()
}

/// Format signatures and definitions of symbols.
pub fn signature_of(module: &Module, rhai: &Rhai, symbol: Symbol) -> String {
    if !module.contains_symbol(symbol) {
        return String::new();
    }

    let sym_data = &module[symbol];

    match &sym_data.kind {
        rhai_hir::symbol::SymbolKind::Fn(f) => sym_data
            .text_range()
            .and_then(|range| {
                if rhai.syntax().text_range().contains_range(range) {
                    Some(rhai.syntax().covering_element(range))
                } else {
                    None
                }
            })
            .and_then(SyntaxElement::into_node)
            .and_then(ExprFn::cast)
            .map(|expr_fn| {
                if f.ty == Type::Unknown {
                    // Format from syntax only.
                    format!(
                        "fn {ident}({params})",
                        ident = &f.name,
                        params = expr_fn
                            .param_list()
                            .map(|param_list| param_list
                                .params()
                                .map(|p| p.ident_token().map(|t| t.to_string()).unwrap_or_default())
                                .collect::<Vec<String>>()
                                .join(","))
                            .unwrap_or_default()
                    )
                } else {
                    format!("fn {ident}{sig:#}", ident = &f.name, sig = &f.ty)
                }
            })
            .unwrap_or_default(),
        rhai_hir::symbol::SymbolKind::Decl(d) => {
            if d.is_param | d.is_pat {
                format!("{name}: {ty:#}", name = d.name.clone(), ty = &d.ty)
            } else {
                format!(
                    "{kw} {ident}: {ty:#}",
                    ident = &d.name,
                    kw = if d.is_const { "const" } else { "let" },
                    ty = &d.ty
                )
            }
        }
        _ => String::new(),
    }
}

pub trait RhaiStringExt {
    fn wrap_rhai_markdown(&self) -> String;
}

impl<T: AsRef<str>> RhaiStringExt for T {
    fn wrap_rhai_markdown(&self) -> String {
        format!("```rhai\n{}\n```", self.as_ref().trim_end())
    }
}
