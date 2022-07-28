use lsp_async_stub::{Context, Params};
use lsp_types::{DidChangeWatchedFilesParams, FileChangeType};

use crate::{
    diagnostics::{clear_diagnostics, publish_all_diagnostics},
    environment::Environment,
    world::World,
};

use super::update_document;

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

                drop(workspaces);
                update_document(context.clone(), uri, &source).await;
            }
            FileChangeType::DELETED => {
                let ws = workspaces.by_document_mut(&uri);
                ws.remove_document(&uri) ;
                clear_diagnostics(context.clone(), uri).await;
            }
            _ => {
                tracing::warn!(change_type = ?change.typ, "unknown file event");
            }
        }
    }

    context
        .clone()
        .all_diagnostics_debouncer
        .spawn(publish_all_diagnostics(context));
}
