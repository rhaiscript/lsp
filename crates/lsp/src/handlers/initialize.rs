use enum_iterator::IntoEnumIterator;

use super::*;

pub(crate) async fn initialize(
    _context: Context<World>,
    _params: Params<InitializeParams>,
) -> Result<InitializeResult, Error> {
    Ok(InitializeResult {
        capabilities: ServerCapabilities {
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
                trigger_characters: Some(vec![
                    "#".into(),
                ]),
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
