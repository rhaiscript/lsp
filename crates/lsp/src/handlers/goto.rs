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

    goto_target(context, uri, pos)
        .map(|result| result.map(|links| GotoDeclarationResponse::Link(links)))
}

// Technically the same, as goto_declaration, but a different function for consistency.
pub(crate) async fn goto_definition(
    context: Context<World>,
    params: Params<GotoDefinitionParams>,
) -> Result<Option<GotoDefinitionResponse>, Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

    goto_target(context, uri, pos)
        .map(|result| result.map(|links| GotoDefinitionResponse::Link(links)))
}

fn goto_target(
    mut context: Context<World>,
    uri: Url,
    pos: Position,
) -> Result<Option<Vec<LocationLink>>, Error> {
    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let offset = match doc.mapper.offset(mapper::Position::from_lsp(pos)) {
        Some(p) => p,
        None => return Ok(None),
    };

    let module = match w.hir.get_module(uri.as_str()) {
        Some(m) => m,
        None => return Ok(None),
    };

    let target_symbol = module
        .symbol_selection_at(offset, true)
        .map(|s| (s, &module[s]));

    if let Some((_, data)) = target_symbol {
        let origin_selection_range = data.selection_syntax.and_then(|s| {
            s.text_range
                .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
        });
        match &data.kind {
            rhai_hir::symbol::SymbolKind::Reference(r) => {
                if let Some(ReferenceTarget::Symbol(target)) = &r.target {
                    let target_data = &module[*target];

                    let target_range = match target_data
                        .text_range()
                        .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
                    {
                        Some(range) => range,
                        None => return Ok(None),
                    };

                    let target_selection_range = target_data
                        .selection_range()
                        .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
                        .unwrap_or(target_range);

                    return Ok(Some(vec![LocationLink {
                        origin_selection_range,
                        target_uri: uri,
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
