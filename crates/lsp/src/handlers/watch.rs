use lsp_async_stub::{Context, Params};
use lsp_types::{DidChangeWatchedFilesParams, FileChangeType};

use crate::{diagnostics::publish_all_diagnostics, environment::Environment, world::World};

use super::{load_missing_documents, update_document};

pub(crate) async fn watched_file_change<E: Environment>(
    context: Context<World<E>>,
    params: Params<DidChangeWatchedFilesParams>,
) {
    let params = match params.optional() {
        Some(p) => p,
        None => return,
    };

    for change in params.changes {
        let uri = change.uri;

        let mut workspaces = context.workspaces.write().await;

        match change.typ {
            FileChangeType::CREATED | FileChangeType::CHANGED => {
                let path = match context.env.url_to_file_path(&uri) {
                    Some(p) => p,
                    None => {
                        tracing::warn!(url = %uri, "could not create file path from url");
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
                        tracing::error!(url = %uri, %error, "source is not valid UTF-8");
                        continue;
                    }
                };

                update_document(&mut *workspaces, &uri, &source);
                let ws = workspaces.by_document_mut(&uri);

                let missing_modules = ws.hir.missing_modules();
                load_missing_documents(context.clone(), &mut *workspaces, missing_modules).await;
            }
            FileChangeType::DELETED => {
                let ws = workspaces.by_document_mut(&uri);
                ws.documents.remove(&uri);

                if let Some(src) = ws.hir.source_by_url(&uri) {
                    ws.hir.remove_source(src);
                }
            }
            _ => {
                tracing::warn!(change_type = ?change.typ, "unknown file event");
            }
        }
    }

    publish_all_diagnostics(context).await;
}
