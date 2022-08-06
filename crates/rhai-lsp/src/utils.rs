use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwapOption;
use futures::{
    future::{AbortHandle, Abortable},
    Future,
};
use rhai_hir::{Hir, Symbol, Type};
use rhai_rowan::{
    ast::{AstNode, ExprFn},
    syntax::{SyntaxElement, SyntaxNode},
};

use rhai_common::environment::Environment;

/// Format signatures and definitions of symbols.
pub fn signature_of(hir: &Hir, root: &SyntaxNode, symbol: Symbol) -> String {
    if hir.symbol(symbol).is_none() {
        return String::new();
    }

    let sym_data = &hir[symbol];

    match &sym_data.kind {
        rhai_hir::symbol::SymbolKind::Fn(f) => sym_data
            .text_range()
            .and_then(|range| {
                if root.text_range().contains_range(range) {
                    Some(root.covering_element(range))
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

pub fn documentation_for(hir: &Hir, root: &SyntaxNode, symbol: Symbol, signature: bool) -> String {
    if let Some(m) = hir.target_module(symbol) {
        return hir[m].docs.clone();
    }

    let sig = if signature {
        signature_of(hir, root, symbol).wrap_rhai_markdown()
    } else {
        String::new()
    };

    let sym_data = &hir[symbol];

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

pub trait RhaiStringExt {
    fn wrap_rhai_markdown(&self) -> String;
}

impl<T: AsRef<str>> RhaiStringExt for T {
    fn wrap_rhai_markdown(&self) -> String {
        format!("```rhai\n{}\n```", self.as_ref().trim_end())
    }
}

pub struct Debouncer<E: Environment> {
    duration: Duration,
    handle: ArcSwapOption<AbortHandle>,
    env: E,
}

impl<E: Environment> Debouncer<E> {
    pub fn new(duration: Duration, env: E) -> Self {
        Self {
            duration,
            handle: Default::default(),
            env,
        }
    }

    pub fn spawn(&self, fut: impl Future + 'static) {
        let prev_handle = self.handle.swap(None);

        if let Some(handle) = prev_handle {
            handle.abort();
        }

        let (abort_handle, abort_reg) = AbortHandle::new_pair();

        let duration = self.duration;
        let env = self.env.clone();

        let fut = Abortable::new(
            async move {
                env.sleep(duration).await;
                fut.await;
            },
            abort_reg,
        );

        self.handle.store(Some(Arc::new(abort_handle)));
        self.env.spawn_local(fut);
    }
}
