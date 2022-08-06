use rhai_common::environment::Environment;
use crate::{
    world::{Workspace, World},
};
use lsp_async_stub::{rpc, util::LspExt, Context, Params};
use lsp_types::{Location, ReferenceParams};
use rhai_hir::Symbol;
use rhai_rowan::{syntax::SyntaxKind, TextRange, TextSize};

pub(crate) async fn references<E: Environment>(
    context: Context<World<E>>,
    params: Params<ReferenceParams>,
) -> Result<Option<Vec<Location>>, rpc::Error> {
    let p = params.required()?;

    let uri = p.text_document_position.text_document.uri;
    let pos = p.text_document_position.position;

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

    let elem = doc
        .parse
        .clone_syntax()
        .covering_element(TextRange::new(offset, offset));

    if elem.kind() != SyntaxKind::IDENT {
        return Ok(None);
    }

    let target_symbol = ws
        .hir
        .symbols()
        .find(|(_, symbol)| symbol.has_selection_range(elem.text_range()));

    if let Some((sym, _)) = target_symbol {
        let mut locations = Vec::new();
        collect_references(ws, sym, p.context.include_declaration, &mut locations);
        return Ok(Some(locations));
    }

    Ok(None)
}

pub(crate) fn collect_references<E: Environment>(
    w: &Workspace<E>,
    target_symbol: Symbol,
    include_declaration: bool,
    locations: &mut Vec<Location>,
) {
    let target_data = &w.hir[target_symbol];

    let references = match &target_data.kind {
        rhai_hir::symbol::SymbolKind::Fn(f) => &f.references,
        rhai_hir::symbol::SymbolKind::Decl(d) => &d.references,
        _ => return,
    };

    locations.extend(
        references
            .iter()
            .filter_map(|&reference| {
                let reference_data = &w.hir[reference];

                let reference_source = match reference_data.source.source {
                    Some(s) => s,
                    None => return None,
                };

                let reference_source_data = &w.hir[reference_source];

                let target_document = match w.documents.get(&reference_source_data.url) {
                    Some(d) => d,
                    None => return None,
                };

                reference_data
                    .source
                    .text_range
                    .and_then(|range| target_document.mapper.range(range).map(LspExt::into_lsp))
                    .map(|range| (reference_source_data.url.clone(), range))
            })
            .map(|(url, range)| Location { uri: url, range }),
    );

    if include_declaration {
        let target_data = &w.hir[target_symbol];

        let target_source = match target_data.source.source {
            Some(s) => s,
            None => return,
        };

        let target_source_data = &w.hir[target_source];

        let target_document = match w.documents.get(&target_source_data.url) {
            Some(d) => d,
            None => return,
        };

        if let Some(range) = target_data
            .source
            .text_range
            .and_then(|range| target_document.mapper.range(range).map(LspExt::into_lsp))
        {
            locations.push(Location {
                uri: target_source_data.url.clone(),
                range,
            });
        }
    }
}
