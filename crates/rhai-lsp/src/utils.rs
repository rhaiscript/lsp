use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwapOption;
use futures::{
    future::{AbortHandle, Abortable},
    Future,
};
use rhai_hir::{symbol::SymbolKind, Hir, Symbol};

use rhai_common::environment::Environment;

/// Format signatures and definitions of symbols.
pub fn signature_of(hir: &Hir, symbol: Symbol) -> String {
    let sym_data = &hir[symbol];

    match &sym_data.kind {
        SymbolKind::Decl(decl) => {
            format!(
                "{}{}: {}",
                if decl.is_param {
                    ""
                } else if decl.is_const {
                    "const "
                } else {
                    "let "
                },
                decl.name,
                sym_data.ty.fmt(hir)
            )
        }
        _ => {
            format!("{}", sym_data.ty.fmt(hir))
        }
    }
}

pub fn documentation_for(hir: &Hir, symbol: Symbol, signature: bool) -> String {
    if let Some(m) = hir.target_module(symbol) {
        return hir[m].docs.clone();
    }

    let sig = if signature {
        signature_of(hir, symbol).wrap_rhai_markdown()
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
