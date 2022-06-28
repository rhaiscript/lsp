use std::path::Path;
use std::sync::Arc;

use super::{update_configuration, update_document};
use crate::config::InitConfig;
use crate::diagnostics::publish_all_diagnostics;
use crate::environment::Environment;
use crate::world::{Workspace, DEFAULT_WORKSPACE_URL};
use crate::World;
use lsp_async_stub::{rpc::Error, Context, Params};
use lsp_types::{
    CompletionOptions, DeclarationCapability, FoldingRangeProviderCapability,
    HoverProviderCapability, InitializedParams, OneOf, RenameOptions, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};
use lsp_types::{InitializeParams, InitializeResult};

#[tracing::instrument(skip_all)]
pub async fn initialize<E: Environment>(
    context: Context<World<E>>,
    params: Params<InitializeParams>,
) -> Result<InitializeResult, Error> {
    let p = params.required()?;

    if let Some(init_opts) = p.initialization_options {
        match serde_json::from_value::<InitConfig>(init_opts) {
            Ok(c) => context.init_config.store(Arc::new(c)),
            Err(error) => {
                tracing::error!(%error, "invalid initialization options");
            }
        }
    }

    if let Some(workspaces) = p.workspace_folders {
        let mut wss = context.workspaces.write().await;

        for workspace in workspaces {
            wss.entry(workspace.uri.clone())
                .or_insert(Workspace::new(context.env.clone(), workspace.uri));
        }
    }

    Ok(InitializeResult {
        capabilities: ServerCapabilities {
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                ..Default::default()
            }),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: Default::default(),
            })),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            references_provider: Some(OneOf::Left(true)),
            declaration_provider: Some(DeclarationCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            // document_formatting_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec!["#".into(), "=".into(), ".".into()]),
                ..CompletionOptions::default()
            }),
            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: "Rhai Language Server".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }),
    })
}

#[tracing::instrument(skip_all)]
pub async fn initialized<E: Environment>(
    context: Context<World<E>>,
    _params: Params<InitializedParams>,
) {
    update_configuration(context.clone()).await;

    let mut workspaces = context.workspaces.write().await;

    let workspace_roots = workspaces
        .iter()
        .filter(|(url, _)| **url != *DEFAULT_WORKSPACE_URL)
        .map(|w| w.0.clone())
        .collect::<Vec<_>>();

    for ws_root in workspace_roots {
        if let Some(root) = context.env.url_to_file_path(&ws_root) {
            tracing::info!(?root, "discovering files");
            let mut count = 0;
            match context.env.rhai_files(&root) {
                Ok(files) => {
                    for file in files {
                        if let Some(path) = file.to_str() {
                            let url: Url = match format!("file://{path}").parse() {
                                Ok(u) => u,
                                Err(_) => continue,
                            };

                            let file_content = match context.env.read_file(Path::new(path)).await {
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

                            count += 1;
                            update_document(&mut *workspaces, &url, &source);
                            tracing::info!(%url, "loaded file");
                        }
                    }

                    tracing::info!(count, ?root, "found files");
                }
                Err(err) => {
                    tracing::warn!(error = %err, "failed to discover files in workspace");
                }
            }
        }
    }

    drop(workspaces);
    publish_all_diagnostics(context).await;
}
