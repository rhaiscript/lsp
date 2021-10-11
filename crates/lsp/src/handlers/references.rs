use crate::mapper::{self, LspExt};

use super::*;
use rhai_rowan::{syntax::SyntaxKind, TextRange, TextSize};

pub(crate) async fn references(
    mut context: Context<World>,
    params: Params<ReferenceParams>,
) -> Result<Option<Vec<Location>>, Error> {
    let p = params.required()?;

    let w = context.world().lock().unwrap();

    let uri = p.text_document_position.text_document.uri;
    let pos = p.text_document_position.position;

    let doc = match w.documents.get(&uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let offset = match doc.mapper.offset(mapper::Position::from_lsp(pos)) {
        Some(p) => p + TextSize::from(1),
        None => return Ok(None),
    };

    let module = match w.hir.get_module(uri.as_str()) {
        Some(m) => m,
        None => return Ok(None),
    };

    let elem = doc
        .parse
        .clone_syntax()
        .covering_element(TextRange::new(offset, offset));

    if elem.kind() != SyntaxKind::IDENT {
        return Ok(None);
    }

    let target_symbol = module
        .symbols()
        .find(|(_, symbol)| symbol.has_selection_range(elem.text_range()));

    if let Some((_, data)) = target_symbol {
        let references = match &data.kind {
            rhai_hir::symbol::SymbolKind::Fn(f) => &f.references,
            rhai_hir::symbol::SymbolKind::Decl(d) => &d.references,
            _ => return Ok(None),
        };

        let mut locations = references
            .iter()
            .filter_map(|&symbol| {
                module[symbol]
                    .syntax
                    .and_then(|syntax| syntax.text_range)
                    .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
            })
            .map(|range: Range| Location {
                uri: uri.clone(),
                range,
            })
            .collect::<Vec<Location>>();

        if p.context.include_declaration {
            if let Some(range) = data
                .syntax
                .and_then(|syntax| syntax.text_range)
                .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
            {
                locations.push(Location {
                    uri: uri.clone(),
                    range,
                });
            }
        }

        return Ok(Some(locations));
    }

    Ok(None)
}
