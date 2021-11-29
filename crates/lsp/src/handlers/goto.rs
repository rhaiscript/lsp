use crate::mapper::{self, LspExt};

use super::*;
use lsp_types::request::{GotoDeclarationParams, GotoDeclarationResponse};
use rhai_hir::symbol::ReferenceTarget;

pub(crate) async fn goto_declaration(
    context: Context<World>,
    params: Params<GotoDeclarationParams>,
) -> Result<Option<GotoDeclarationResponse>, Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

    goto_target(context, uri, pos).map(|result| result.map(GotoDeclarationResponse::Link))
}

// Technically the same, as goto_declaration, but a different function for consistency.
pub(crate) async fn goto_definition(
    context: Context<World>,
    params: Params<GotoDefinitionParams>,
) -> Result<Option<GotoDefinitionResponse>, Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

    goto_target(context, uri, pos).map(|result| result.map(GotoDefinitionResponse::Link))
}

fn goto_target(
    mut context: Context<World>,
    uri: Url,
    pos: Position,
) -> Result<Option<Vec<LocationLink>>, Error> {
    let w = context.world().read();

    let doc = match w.documents.get(&uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let offset = match doc.mapper.offset(mapper::Position::from_lsp(pos)) {
        Some(p) => p,
        None => return Ok(None),
    };

    let source = match w.hir.source_of(&uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    let target_symbol = w
        .hir
        .symbol_selection_at(source, offset, true)
        .map(|s| (s, &w.hir[s]));

    if let Some((_, data)) = target_symbol {
        let origin_selection_range = data
            .selection_or_text_range()
            .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp));
        match &data.kind {
            rhai_hir::symbol::SymbolKind::Reference(r) => {
                if let Some(ReferenceTarget::Symbol(target)) = &r.target {
                    let target_data = &w.hir[*target];

                    let target_source = match target_data.source.source {
                        Some(s) => s,
                        None => return Ok(None),
                    };

                    let target_source_data = &w.hir[target_source];

                    let target_document = match w.documents.get(&target_source_data.url) {
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
                            .map_or(uri, |s| w.hir[s].url.clone()),
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
