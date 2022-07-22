use lsp_async_stub::{util::Mapper, Context, Params};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Url,
};
use rhai_rowan::{parser::Parser, util::is_rhai_def};

use crate::{
    diagnostics::{publish_all_diagnostics, publish_diagnostics},
    environment::Environment,
    utils::Normalize,
    world::{Document, World},
};

#[tracing::instrument(skip_all)]
pub(crate) async fn document_open<E: Environment>(
    context: Context<World<E>>,
    params: Params<DidOpenTextDocumentParams>,
) {
    let p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    update_document(
        context.clone(),
        p.text_document.uri.clone(),
        &p.text_document.text,
    )
    .await;
    publish_diagnostics(context.clone(), p.text_document.uri).await;

    context
        .clone()
        .all_diagnostics_debouncer
        .spawn(publish_all_diagnostics(context));
}

#[tracing::instrument(skip_all)]
pub(crate) async fn document_change<E: Environment>(
    context: Context<World<E>>,
    params: Params<DidChangeTextDocumentParams>,
) {
    let mut p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    // We expect one full change
    let change = match p.content_changes.pop() {
        None => return,
        Some(c) => c,
    };

    update_document(context.clone(), p.text_document.uri.clone(), &change.text).await;
    publish_diagnostics(context.clone(), p.text_document.uri).await;

    context
        .clone()
        .all_diagnostics_debouncer
        .spawn(publish_all_diagnostics(context));
}

#[tracing::instrument(skip_all)]
pub(crate) async fn document_save<E: Environment>(
    _context: Context<World<E>>,
    _params: Params<DidSaveTextDocumentParams>,
) {
    // stub to silence warnings
}

#[tracing::instrument(skip_all)]
pub(crate) async fn document_close<E: Environment>(
    _context: Context<World<E>>,
    _params: Params<DidCloseTextDocumentParams>,
) {
    // no-op, we track it until it is deleted.
}

#[tracing::instrument(skip_all)]
pub(crate) async fn update_document<E: Environment>(ctx: Context<World<E>>, uri: Url, text: &str) {
    let parse = if is_rhai_def(text) {
        Parser::new(text).parse_def()
    } else {
        Parser::new(text).parse_script()
    };

    let uri = uri.normalize();

    let mapper = Mapper::new_utf16(text, false);

    let mut ws = ctx.workspaces.write().await;
    let ws = ws.by_document_mut(&uri);

    ws.hir.add_source(&uri, &parse.clone_syntax());
    ws.hir.resolve_references();
    ws.documents.insert(uri.clone(), Document { parse, mapper });
}
