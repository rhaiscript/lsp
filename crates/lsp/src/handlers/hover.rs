use crate::{
    mapper::{self, LspExt},
    util::documentation_for,
};
use rhai_hir::{symbol::ReferenceTarget, Hir, Symbol};
use rhai_rowan::ast::{AstNode, Rhai};

use super::*;

pub(crate) async fn hover(
    mut context: Context<World>,
    params: Params<HoverParams>,
) -> Result<Option<Hover>, Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

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

    let rhai = match Rhai::cast(doc.parse.clone_syntax()) {
        Some(r) => r,
        None => return Ok(None),
    };

    let target_symbol = w
        .hir
        .symbol_selection_at(source, offset, true)
        .map(|s| (s, &w.hir[s]));

    if let Some((symbol, data)) = target_symbol {
        let highlight_range = data
            .selection_or_text_range()
            .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp));

        return Ok(hover_for_symbol(&w.hir, &rhai, highlight_range, symbol));
    }

    Ok(None)
}

fn hover_for_symbol(
    hir: &Hir,
    rhai: &Rhai,
    highlight_range: Option<Range>,
    symbol: Symbol,
) -> Option<Hover> {
    match &hir[symbol].kind {
        rhai_hir::symbol::SymbolKind::Fn(_) | rhai_hir::symbol::SymbolKind::Decl(_) => {
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, rhai, symbol, true),
                }),
                range: highlight_range,
            })
        }
        rhai_hir::symbol::SymbolKind::Reference(r) => match &r.target {
            Some(ReferenceTarget::Symbol(target)) => {
                hover_for_symbol(hir, rhai, highlight_range, *target)
            }
            _ => None,
        },
        _ => None,
    }
}
