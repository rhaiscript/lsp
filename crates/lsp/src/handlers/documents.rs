use lsp_async_stub::{util::Mapper, Context, Params};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Url,
};
use rhai_rowan::{parser::Parser, util::is_rhai_def};

use crate::{
    diagnostics::publish_all_diagnostics,
    environment::Environment,
    world::{Document, Workspaces, World},
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

    let mut workspaces = context.workspaces.write().await;
    update_document(
        &mut *workspaces,
        &p.text_document.uri,
        &p.text_document.text,
    );

    let ws = workspaces.by_document(&p.text_document.uri);

    let mut missing_modules = ws.hir.missing_modules();

    // TODO: proper strategy for loading modules.
    let mut last_len = 0;

    while missing_modules.len() > 0 {
        if last_len == missing_modules.len() {
            break;
        }

        last_len = missing_modules.len();

        load_missing_documents(context.clone(), &mut *workspaces, missing_modules).await;
        let ws = workspaces.by_document(&p.text_document.uri);
        missing_modules = ws.hir.missing_modules();
    }

    load_missing_documents(context.clone(), &mut *workspaces, missing_modules).await;
    drop(workspaces);
    publish_all_diagnostics(context).await;
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

    let mut workspaces = context.workspaces.write().await;
    update_document(&mut *workspaces, &p.text_document.uri, &change.text);

    let ws = workspaces.by_document(&p.text_document.uri);

    let mut missing_modules = ws.hir.missing_modules();

    // TODO: proper strategy for loading modules.
    let mut last_len = 0;

    while missing_modules.len() > 0 {
        if last_len == missing_modules.len() {
            break;
        }

        last_len = missing_modules.len();

        load_missing_documents(context.clone(), &mut *workspaces, missing_modules).await;
        let ws = workspaces.by_document(&p.text_document.uri);
        missing_modules = ws.hir.missing_modules();
    }

    drop(workspaces);
    publish_all_diagnostics(context).await;
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

pub(crate) fn update_document<E: Environment>(
    workspaces: &mut Workspaces<E>,
    uri: &Url,
    text: &str,
) {
    let parse = if is_rhai_def(text) {
        Parser::new(text).parse_def()
    } else {
        Parser::new(text).parse_script()
    };

    let mapper = Mapper::new_utf16(text, false);

    let ws = workspaces.by_document_mut(uri);

    ws.hir.add_source(uri, &parse.clone_syntax());
    ws.hir.resolve_references();
    ws.documents.insert(uri.clone(), Document { parse, mapper });
}

#[tracing::instrument(skip_all)]
pub(crate) async fn load_missing_documents<E: Environment>(
    context: Context<World<E>>,
    workspaces: &mut Workspaces<E>,
    urls: impl Iterator<Item = Url>,
) {
    for url in urls {
        let path = match context.env.url_to_file_path(&url) {
            Some(p) => p,
            None => {
                tracing::warn!(%url, "could not create file path from url");
                continue;
            }
        };

        let file_content = match context.env.read_file(&path).await {
            Ok(c) => c,
            Err(err) => {
                tracing::error!(error = %err, "failed to read file");
                continue;
            }
        };

        let source = match String::from_utf8(file_content) {
            Ok(s) => s,
            Err(error) => {
                tracing::error!(%url, %error, "source is not valid UTF-8");
                continue;
            }
        };

        update_document(workspaces, &url, &source);
    }
}
