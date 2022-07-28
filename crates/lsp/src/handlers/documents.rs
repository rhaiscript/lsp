use lsp_async_stub::{Context, Params};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Url,
};

use crate::{
    diagnostics::{publish_all_diagnostics, publish_diagnostics},
    environment::Environment,
    world::World,
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
    let mut ws = ctx.workspaces.write().await;
    let ws = ws.by_document_mut(&uri);
    ws.add_document(uri, text);
    ws.hir.resolve_references();
}
