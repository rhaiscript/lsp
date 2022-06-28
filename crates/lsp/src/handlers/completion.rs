use crate::{
    environment::Environment,
    utils::{documentation_for, signature_of},
    world::World,
};
use itertools::Itertools;
use lsp_async_stub::{
    rpc,
    util::{LspExt, Position},
    Context, Params,
};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind,
};
use rhai_hir::{symbol::SymbolKind, Hir, Symbol};
use rhai_rowan::{query::Query, syntax::SyntaxNode};

pub(crate) async fn completion<E: Environment>(
    context: Context<World<E>>,
    params: Params<CompletionParams>,
) -> Result<Option<CompletionResponse>, rpc::Error> {
    let p = params.required()?;

    let uri = p.text_document_position.text_document.uri;
    let pos = p.text_document_position.position;

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&uri);

    let doc = ws.document(&uri)?;

    let syntax = doc.parse.clone().into_syntax();

    let offset = match doc.mapper.offset(Position::from_lsp(pos)) {
        Some(p) => p,
        None => return Ok(None),
    };

    let source = match ws.hir.source_of(&uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    let query = Query::at(&syntax, offset);

    if query.is_path() {
        let modules = ws
            .hir
            .visible_symbols_from_offset(source, offset, false)
            .filter_map(|symbol| ws.hir[symbol].kind.as_import().and_then(|d| d.alias));

        let idx = query.path_segment_index();

        if idx == 0 {
            return Ok(Some(CompletionResponse::Array(
                modules
                    .filter_map(|symbol| reference_completion(&ws.hir, &syntax, true, symbol))
                    .unique_by(|(symbol, _)| ws.hir.unique_symbol_name(symbol))
                    .map(|(_, c)| c)
                    .collect(),
            )));
        }

        let mut symbols = modules.collect::<Vec<_>>();

        for (i, segment) in query.path().unwrap().segments().enumerate() {
            let module_name = segment.text();

            let module_symbol = symbols
                .iter()
                .find(|&&symbol| ws.hir[symbol].name() == Some(module_name));

            let module_symbol = match module_symbol {
                Some(s) => *s,
                None => break,
            };

            match ws.hir.target_module(module_symbol) {
                Some(m) => {
                    symbols = ws
                        .hir
                        .descendant_symbols(ws.hir[m].scope)
                        .filter(|s| ws.hir[*s].export)
                        .collect();
                }
                None => break,
            }

            if i == idx {
                break;
            }
        }

        Ok(Some(CompletionResponse::Array(
            symbols
                .into_iter()
                .filter_map(|symbol| reference_completion(&ws.hir, &syntax, false, symbol))
                .unique_by(|(symbol, _)| ws.hir.unique_symbol_name(symbol))
                .map(|(_, c)| c)
                .collect(),
        )))
    } else {
        Ok(Some(CompletionResponse::Array(
            ws.hir
                .visible_symbols_from_offset(source, offset, false)
                .filter_map(|symbol| {
                    // Unwrap aliases from import symbols
                    ws.hir[symbol]
                        .kind
                        .as_import()
                        .and_then(|d| d.alias)
                        .or(Some(symbol))
                })
                .filter_map(|symbol| reference_completion(&ws.hir, &syntax, false, symbol))
                .unique_by(|(symbol, _)| ws.hir.unique_symbol_name(symbol))
                .map(|(_, c)| c)
                .collect(),
        )))
    }
}

fn reference_completion(
    hir: &Hir,
    syntax: &SyntaxNode,
    ident_only: bool,
    symbol: Symbol,
) -> Option<(Symbol, CompletionItem)> {
    match &hir[symbol].kind {
        SymbolKind::Fn(f) => Some((
            symbol,
            CompletionItem {
                label: f.name.clone(),
                detail: Some(signature_of(hir, syntax, symbol)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, syntax, symbol, false),
                })),
                kind: Some(CompletionItemKind::FUNCTION),
                insert_text: Some(format!("{}($0)", &f.name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..CompletionItem::default()
            },
        )),
        SymbolKind::Decl(d) => Some((
            symbol,
            CompletionItem {
                label: d.name.clone(),
                detail: Some(signature_of(hir, syntax, symbol)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, syntax, symbol, false),
                })),
                kind: Some(if d.is_const {
                    CompletionItemKind::CONSTANT
                } else if d.is_import {
                    CompletionItemKind::MODULE
                } else {
                    CompletionItemKind::VARIABLE
                }),
                insert_text: if d.is_import && !ident_only {
                    Some(format!("{}::", d.name))
                } else {
                    Some(d.name.clone())
                },
                ..CompletionItem::default()
            },
        )),
        _ => None,
    }
}
