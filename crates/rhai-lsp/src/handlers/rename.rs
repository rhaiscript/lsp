use crate::world::{Workspace, World};
use core::iter;
use lsp_async_stub::{
    rpc::Error,
    util::{LspExt, Position},
    Context, Params,
};
use lsp_types::{
    PrepareRenameResponse, RenameParams, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit,
};
use rhai_common::{environment::Environment, util::Normalize};
use rhai_hir::{symbol, Hir, Symbol};
use std::collections::HashMap;

#[tracing::instrument(skip_all)]
pub async fn prepare_rename<E: Environment>(
    context: Context<World<E>>,
    params: Params<TextDocumentPositionParams>,
) -> Result<Option<PrepareRenameResponse>, Error> {
    let p = params.required()?;
    let document_uri = p.text_document.uri;

    let workspaces = context.workspaces.write().await;
    let ws = workspaces.by_document(&document_uri);
    let doc = ws.document(&document_uri)?;

    let position = p.position;
    let offset = match doc.mapper.offset(Position::from_lsp(position)) {
        Some(ofs) => ofs,
        None => {
            tracing::error!(?position, "document position not found");
            return Ok(None);
        }
    };

    let source = match ws.hir.source_of(&document_uri.clone().normalize()) {
        Some(s) => s,
        None => return Ok(None),
    };

    let target_symbol = ws
        .hir
        .symbol_selection_at(source, offset, true)
        .map(|s| (s, &ws.hir[s]));

    Ok(target_symbol.and_then(|(_, data)| {
        let range = data.selection_range().and_then(|r| doc.mapper.range(r));

        match range {
            Some(range) => match &data.kind {
                symbol::SymbolKind::Fn(_)
                | symbol::SymbolKind::Decl(_)
                | symbol::SymbolKind::Ref(_) => {
                    Some(PrepareRenameResponse::Range(range.into_lsp()))
                }
                _ => None,
            },
            None => None,
        }
    }))
}

#[tracing::instrument(skip_all)]
pub async fn rename<E: Environment>(
    context: Context<World<E>>,
    params: Params<RenameParams>,
) -> Result<Option<WorkspaceEdit>, Error> {
    let p = params.required()?;
    let document_uri = p.text_document_position.text_document.uri;

    let workspaces = context.workspaces.write().await;
    let ws = workspaces.by_document(&document_uri);
    let doc = ws.document(&document_uri)?;

    let position = p.text_document_position.position;
    let offset = match doc.mapper.offset(Position::from_lsp(position)) {
        Some(ofs) => ofs,
        None => {
            tracing::error!(?position, "document position not found");
            return Ok(None);
        }
    };

    let source = match ws.hir.source_of(&document_uri.clone().normalize()) {
        Some(s) => s,
        None => return Ok(None),
    };

    let mut target_symbol = ws
        .hir
        .symbol_selection_at(source, offset, true)
        .map(|s| (s, &ws.hir[s]));

    if let Some((sym, data)) = target_symbol.take() {
        if let symbol::SymbolKind::Ref(r) = &data.kind {
            match r.target {
                Some(t) => match t {
                    symbol::ReferenceTarget::Symbol(ts) => {
                        target_symbol = Some((ts, &ws.hir[ts]));
                    }
                    symbol::ReferenceTarget::Module(_) => {
                        tracing::warn!("renaming a module is not possible");
                    }
                },
                None => {}
            }
        } else {
            target_symbol = Some((sym, data));
        }
    }

    Ok(
        target_symbol.and_then(|(target_symbol, data)| match &data.kind {
            symbol::SymbolKind::Fn(target) => Some(WorkspaceEdit {
                changes: Some(rename_symbols(
                    &ws.hir,
                    iter::once(target_symbol).chain(target.references.iter().copied()),
                    &p.new_name,
                    ws,
                )),
                ..Default::default()
            }),
            symbol::SymbolKind::Decl(target) => Some(WorkspaceEdit {
                changes: Some(rename_symbols(
                    &ws.hir,
                    iter::once(target_symbol).chain(target.references.iter().copied()),
                    &p.new_name,
                    ws,
                )),
                ..Default::default()
            }),
            _ => None,
        }),
    )
}

fn rename_symbols<E: Environment>(
    hir: &Hir,
    symbols: impl Iterator<Item = Symbol>,
    new_name: &str,
    ws: &Workspace<E>,
) -> HashMap<Url, Vec<TextEdit>> {
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    for symbol in symbols {
        let data = &hir[symbol];

        let source = match data.source.source {
            Some(s) => s,
            None => continue,
        };

        let range = match data.source.selection_text_range {
            Some(range) => range,
            None => continue,
        };

        let url = &hir[source].url;

        let doc = match ws.document(url) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let document_edits = changes.entry(url.clone()).or_default();

        if let Some(range) = doc.mapper.range(range) {
            document_edits.push(TextEdit {
                new_text: new_name.into(),
                range: range.into_lsp(),
            });
        }
    }

    changes
}
