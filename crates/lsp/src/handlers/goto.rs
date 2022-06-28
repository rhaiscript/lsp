use crate::{environment::Environment, world::World};

use lsp_async_stub::{rpc, Context, Params, util::LspExt};
use lsp_types::{request::{GotoDeclarationParams, GotoDeclarationResponse}, Url, Position, LocationLink, GotoDefinitionParams, GotoDefinitionResponse};
use rhai_hir::symbol::ReferenceTarget;

pub(crate) async fn goto_declaration<E: Environment>(
    context: Context<World<E>>,
    params: Params<GotoDeclarationParams>,
) -> Result<Option<GotoDeclarationResponse>, rpc::Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

    goto_target(context, uri, pos).await.map(|result| result.map(GotoDeclarationResponse::Link))
}

// Technically the same, as goto_declaration, but a different function for consistency.
pub(crate) async fn goto_definition<E: Environment>(
    context: Context<World<E>>,
    params: Params<GotoDefinitionParams>,
) -> Result<Option<GotoDefinitionResponse>, rpc::Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

    goto_target(context, uri, pos).await.map(|result| result.map(GotoDefinitionResponse::Link))
}

async fn goto_target<E: Environment>(
    context: Context<World<E>>,
    uri: Url,
    pos: Position,
) -> Result<Option<Vec<LocationLink>>, rpc::Error> {
    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&uri);

    let doc = ws.document(&uri)?;

    let offset = match doc.mapper.offset(lsp_async_stub::util::Position::from_lsp(pos)) {
        Some(p) => p,
        None => return Ok(None),
    };

    let source = match ws.hir.source_of(&uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    let target_symbol = ws
        .hir
        .symbol_selection_at(source, offset, true)
        .map(|s| (s, &ws.hir[s]));

    if let Some((_, data)) = target_symbol {
        let origin_selection_range = data
            .selection_or_text_range()
            .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp));
        match &data.kind {
            rhai_hir::symbol::SymbolKind::Reference(r) => {
                if let Some(ReferenceTarget::Symbol(target)) = &r.target {
                    let target_data = &ws.hir[*target];

                    let target_source = match target_data.source.source {
                        Some(s) => s,
                        None => return Ok(None),
                    };

                    let target_source_data = &ws.hir[target_source];

                    let target_document = match ws.documents.get(&target_source_data.url) {
                        Some(d) => d,
                        None => return Ok(None),
                    };

                    let target_range = match target_data
                        .text_range()
                        .and_then(|range| target_document.mapper.range(range).map(LspExt::into_lsp))
                    {
                        Some(range) => range,
                        None => return Ok(None),
                    };

                    let target_selection_range = target_data
                        .selection_range()
                        .and_then(|range| target_document.mapper.range(range).map(LspExt::into_lsp))
                        .unwrap_or(target_range);

                    return Ok(Some(vec![LocationLink {
                        origin_selection_range,
                        target_uri: target_data
                            .source
                            .source
                            .map_or(uri, |s| ws.hir[s].url.clone()),
                        target_range,
                        target_selection_range,
                    }]));
                }
            }
            _ => {}
        }
    }

    Ok(None)
}
