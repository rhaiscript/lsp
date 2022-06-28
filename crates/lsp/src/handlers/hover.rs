use crate::{environment::Environment, utils::documentation_for, world::World};
use lsp_async_stub::{rpc, util::LspExt, Context, Params};
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Range};
use rhai_hir::{symbol::ReferenceTarget, Hir, Symbol};
use rhai_rowan::{syntax::SyntaxNode, TextSize};

pub(crate) async fn hover<E: Environment>(
    context: Context<World<E>>,
    params: Params<HoverParams>,
) -> Result<Option<Hover>, rpc::Error> {
    let p = params.required()?;

    let uri = p.text_document_position_params.text_document.uri;
    let pos = p.text_document_position_params.position;

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&uri);

    let doc = ws.document(&uri)?;

    let offset = match doc
        .mapper
        .offset(lsp_async_stub::util::Position::from_lsp(pos))
    {
        Some(p) => p + TextSize::from(1),
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

    if let Some((symbol, data)) = target_symbol {
        let highlight_range = data
            .selection_or_text_range()
            .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp));

        return Ok(hover_for_symbol(
            &ws.hir,
            &doc.parse.clone_syntax(),
            highlight_range,
            symbol,
        ));
    }

    Ok(None)
}

fn hover_for_symbol(
    hir: &Hir,
    root: &SyntaxNode,
    highlight_range: Option<Range>,
    symbol: Symbol,
) -> Option<Hover> {
    match &hir[symbol].kind {
        rhai_hir::symbol::SymbolKind::Fn(_) | rhai_hir::symbol::SymbolKind::Decl(_) => {
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, root, symbol, true),
                }),
                range: highlight_range,
            })
        }
        rhai_hir::symbol::SymbolKind::Reference(r) => match &r.target {
            Some(ReferenceTarget::Symbol(target)) => {
                hover_for_symbol(hir, root, highlight_range, *target)
            }
            _ => None,
        },
        _ => None,
    }
}
