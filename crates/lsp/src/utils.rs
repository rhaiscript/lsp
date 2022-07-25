use std::{
    borrow::Cow,
    mem,
    path::{Path, PathBuf},
    time::Duration, sync::Arc,
};

use arc_swap::ArcSwapOption;
use futures::{
    future::{AbortHandle, Abortable},
    Future,
};
use globset::{Glob, GlobSetBuilder};
use lsp_types::Url;
use percent_encoding::percent_decode_str;
use rhai_hir::{Hir, Symbol, Type};
use rhai_rowan::{
    ast::{AstNode, ExprFn},
    syntax::{SyntaxElement, SyntaxNode},
};

use crate::environment::Environment;

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

#[derive(Debug, Clone)]
pub struct GlobRule {
    include: globset::GlobSet,
    exclude: globset::GlobSet,
}

impl GlobRule {
    pub fn new(
        include: impl IntoIterator<Item = impl AsRef<str>>,
        exclude: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, anyhow::Error> {
        let mut inc = GlobSetBuilder::new();
        for glob in include {
            inc.add(Glob::new(glob.as_ref())?);
        }

        let mut exc = GlobSetBuilder::new();
        for glob in exclude {
            exc.add(Glob::new(glob.as_ref())?);
        }

        Ok(Self {
            include: inc.build()?,
            exclude: exc.build()?,
        })
    }

    pub fn is_match(&self, text: impl AsRef<Path>) -> bool {
        if !self.include.is_match(text.as_ref()) {
            return false;
        }

        !self.exclude.is_match(text.as_ref())
    }
}

pub trait Normalize {
    /// Normalizing in the context of this library means the following:
    ///
    /// - replace `\` with `/` on windows
    /// - decode all percent-encoded characters
    #[must_use]
    fn normalize(self) -> Self;
}

impl Normalize for PathBuf {
    fn normalize(self) -> Self {
        match self.to_str() {
            Some(s) => (*normalize_str(s)).into(),
            None => self,
        }
    }
}

impl Normalize for Vec<PathBuf> {
    fn normalize(mut self) -> Self {
        for p in &mut self {
            *p = mem::take(p).normalize();
        }
        self
    }
}

impl Normalize for Url {
    fn normalize(self) -> Self {
        if self.scheme() != "file" {
            return self;
        }

        if let Ok(u) = normalize_str(self.as_str()).parse() {
            return u;
        }

        self
    }
}

pub(crate) fn normalize_str(s: &str) -> Cow<str> {
    let percent_decoded = match percent_decode_str(s).decode_utf8().ok() {
        Some(s) => s,
        None => return s.into(),
    };

    if cfg!(windows) {
        percent_decoded.replace('\\', "/").into()
    } else {
        percent_decoded
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
