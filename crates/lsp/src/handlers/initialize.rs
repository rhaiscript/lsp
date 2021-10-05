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
            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: "rhai-lsp".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }),
    })
}
