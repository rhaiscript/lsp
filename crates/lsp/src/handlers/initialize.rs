use enum_iterator::IntoEnumIterator;
use lsp_async_stub::RequestWriter;
use lsp_types::request::WorkspaceFoldersRequest;

use crate::watcher::WorkspaceWatcher;

use super::*;

pub(crate) async fn initialize(
    context: Context<World>,
    _params: Params<InitializeParams>,
) -> Result<InitializeResult, Error> {
    let mut ctx = context.clone();
    context
        .defer(async move {
            if let Ok(res) = ctx
                .write_request::<WorkspaceFoldersRequest, _>(Some(()))
                .await
            {
                if let Ok(Some(folders)) = res.into_result() {
                    tracing::info!(?folders, "workspace folders");

                    let c = ctx.clone();
                    let w = ctx.world().read();
                    let watcher = match w.watcher.get_or_try_init(move || WorkspaceWatcher::new(c))
                    {
                        Ok(w) => w,
                        Err(err) => {
                            tracing::error!(error = %err, "failed to initialize workspace watcher");
                            return;
                        }
                    };

                    for folder in folders {
                        if let Ok(p) = folder.uri.to_file_path() {
                            watcher.add_workspace(&p);
                        }
                    }
                }
            }
        })
        .await;

    Ok(InitializeResult {
        capabilities: ServerCapabilities {
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                ..Default::default()
            }),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
            document_symbol_provider: Some(OneOf::Left(true)),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            declaration_provider: Some(DeclarationCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    legend: SemanticTokensLegend {
                        token_types: SemanticTokenKind::into_enum_iter()
                            .map(Into::into)
                            .collect(),
                        token_modifiers: SemanticTokenModifierKind::into_enum_iter()
                            .map(Into::into)
                            .collect(),
                    },
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    range: None,
                    ..SemanticTokensOptions::default()
                }),
            ),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec!["#".into()]),
                ..CompletionOptions::default()
            }),
            ..ServerCapabilities::default()
        },
        server_info: Some(ServerInfo {
            name: "rhai-lsp".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }),
    })
}
