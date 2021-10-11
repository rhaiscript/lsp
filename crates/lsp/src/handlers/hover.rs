use crate::{
    mapper::{self, LspExt},
    util::documentation_for,
};
use rhai_hir::{symbol::ReferenceTarget, Module, Symbol};
use rhai_rowan::ast::{AstNode, Rhai};

use super::*;

pub(crate) async fn hover(
    mut context: Context<World>,
    params: Params<HoverParams>,
) -> Result<Option<Hover>, Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

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

    let rhai = match Rhai::cast(doc.parse.clone_syntax()) {
        Some(r) => r,
        None => return Ok(None),
    };

    let target_symbol = module
        .symbol_selection_at(offset, true)
        .map(|s| (s, &module[s]));

    if let Some((symbol, data)) = target_symbol {
        let highlight_range = data.selection_syntax.and_then(|s| {
            s.text_range
                .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
        });

        return Ok(hover_for_symbol(module, &rhai, highlight_range, symbol));
    }

    Ok(None)
}

fn hover_for_symbol(
    module: &Module,
    rhai: &Rhai,
    highlight_range: Option<Range>,
    symbol: Symbol,
) -> Option<Hover> {
    match &module[symbol].kind {
        rhai_hir::symbol::SymbolKind::Fn(_) | rhai_hir::symbol::SymbolKind::Decl(_) => Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: documentation_for(module, rhai, symbol, true),
            }),
            range: highlight_range,
        }),
        rhai_hir::symbol::SymbolKind::Reference(r) => match &r.target {
            Some(ReferenceTarget::Symbol(target)) => {
                hover_for_symbol(module, rhai, highlight_range, *target)
            }
            _ => None,
        },
        _ => None,
    }
}
