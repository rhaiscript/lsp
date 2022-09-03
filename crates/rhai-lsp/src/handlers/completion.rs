use crate::{
    utils::{documentation_for, signature_of},
    world::{Document, Workspace, World},
};
use itertools::Itertools;
use lsp_async_stub::{
    rpc,
    util::{LspExt, Position},
    Context, Params,
};
use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    CompletionTextEdit, Documentation, InsertTextFormat, MarkupContent, MarkupKind, TextEdit,
};
use rhai_common::{environment::Environment, util::Normalize};
use rhai_hir::{
    scope::ScopeParent,
    symbol::{ReferenceTarget, SymbolKind, VirtualSymbol},
    ty::Type,
    Hir, Symbol, TypeKind,
};
use rhai_rowan::{query::Query, TextRange};

#[tracing::instrument(skip_all)]
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

    let source = match ws.hir.source_of(&uri.clone().normalize()) {
        Some(s) => s,
        None => return Ok(None),
    };

    let query = Query::at(&syntax, offset);

    if query.is_in_comment() {
        return Ok(None);
    }

    if query.is_field_access() {
        if let Some(sym) = ws.hir.symbol_at(source, offset, true) {
            let sym_data = &ws.hir[sym];
            match &sym_data.kind {
                SymbolKind::Binary(b) => Ok(binary_field_access_completion(b, ws, doc, &query)),
                _ => {
                    if let Some(b) = ws.hir[sym_data.parent_scope]
                        .parent
                        .as_ref()
                        .and_then(ScopeParent::as_symbol)
                        .and_then(|&sym| ws.hir[sym].kind.as_binary())
                    {
                        Ok(binary_field_access_completion(b, ws, doc, &query))
                    } else {
                        Ok(None)
                    }
                }
            }
        } else {
            Ok(None)
        }
    } else if query.is_path() {
        let modules = ws
            .hir
            .visible_symbols_from_offset(source, offset, false)
            .filter_map(|symbol| {
                ws.hir[symbol]
                    .kind
                    .as_import()
                    .and_then(|d| d.alias)
                    .or_else(|| {
                        if ws.hir[symbol]
                            .kind
                            .as_virtual()
                            .map_or(false, VirtualSymbol::is_module)
                        {
                            Some(symbol)
                        } else {
                            None
                        }
                    })
            });

        let idx = query.path_segment_index();

        if idx == 0 {
            return Ok(Some(CompletionResponse::Array(
                modules
                    .filter_map(|symbol| reference_completion(&ws.hir, true, symbol))
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
                .find(|&&symbol| ws.hir[symbol].name(&ws.hir) == Some(module_name));

            let module_symbol = match module_symbol {
                Some(s) => *s,
                None => break,
            };

            match ws.hir.target_module(module_symbol) {
                Some(m) => {
                    symbols = ws
                        .hir
                        .scope_symbols(ws.hir[m].scope)
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
                .filter_map(|symbol| reference_completion(&ws.hir, false, symbol))
                .unique_by(|(symbol, _)| ws.hir.unique_symbol_name(symbol))
                .map(|(_, c)| c)
                .collect(),
        )))
    } else if query.can_complete_ref() {
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
                .filter_map(|symbol| reference_completion(&ws.hir, false, symbol))
                .unique_by(|(symbol, _)| ws.hir.unique_symbol_name(symbol))
                .map(|(_, c)| c)
                .collect(),
        )))
    } else if query.can_complete_op() {
        Ok(Some(CompletionResponse::Array(
            ws.hir
                .operators()
                .unique_by(|n| n.name.clone())
                .map(|op| {
                    let text_edit = query.binary_op_ident().map(|ident| {
                        CompletionTextEdit::Edit(TextEdit {
                            new_text: format!("{} ", op.name),
                            range: doc.mapper.range(ident.text_range()).unwrap().into_lsp(),
                        })
                    });

                    CompletionItem {
                        label: op.name.clone(),
                        detail: Some(op.signature(&ws.hir)),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: op.docs.clone(),
                        })),
                        kind: Some(CompletionItemKind::OPERATOR),
                        insert_text: Some(format!("{} ", &op.name)),
                        command: text_edit.is_none().then(trigger_completion),
                        text_edit,
                        ..CompletionItem::default()
                    }
                })
                .collect(),
        )))
    } else {
        Ok(None)
    }
}

fn binary_field_access_completion<E: Environment>(
    b: &rhai_hir::symbol::BinarySymbol,
    ws: &Workspace<E>,
    doc: &Document,
    query: &Query,
) -> std::option::Option<lsp_types::CompletionResponse> {
    if let Some(lhs_ty) = b.lhs.map(|lhs| ws.hir[lhs].ty) {
        let lhs_ty_data = &ws.hir[lhs_ty];

        match &lhs_ty_data.kind {
            TypeKind::Object(o) => Some(CompletionResponse::Array(
                o.fields
                    .iter()
                    .map(|(name, ty)| {
                        field_completion(
                            doc,
                            &ws.hir,
                            name,
                            *ty,
                            query.ident().map(|t| t.text_range()),
                        )
                    })
                    .collect(),
            )),
            _ => {
                // TODO: handle the rest of the types,
                // functions with getters and known `this` type.
                None
            }
        }
    } else {
        None
    }
}

fn reference_completion(
    hir: &Hir,
    ident_only: bool,
    symbol: Symbol,
) -> Option<(Symbol, CompletionItem)> {
    match &hir[symbol].kind {
        SymbolKind::Fn(f) => Some((
            symbol,
            CompletionItem {
                label: f.name.clone(),
                detail: Some(signature_of(hir, symbol)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, symbol, false),
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
                detail: Some(signature_of(hir, symbol)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, symbol, false),
                })),
                kind: Some(if d.is_const {
                    CompletionItemKind::CONSTANT
                } else if d.is_import {
                    CompletionItemKind::MODULE
                } else {
                    CompletionItemKind::VARIABLE
                }),
                insert_text: if let Some(ReferenceTarget::Module(m)) = d.target {
                    if ident_only || hir[hir[m].scope].is_empty() {
                        Some(d.name.clone())
                    } else {
                        Some(format!("{}::", d.name))
                    }
                } else {
                    Some(d.name.clone())
                },
                command: d.is_import.then(trigger_completion),
                ..CompletionItem::default()
            },
        )),
        SymbolKind::Virtual(VirtualSymbol::Module(m)) => Some((
            symbol,
            CompletionItem {
                label: m.name.clone(),
                detail: Some(signature_of(hir, symbol)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: documentation_for(hir, symbol, false),
                })),
                kind: Some(CompletionItemKind::MODULE),
                insert_text: if ident_only || hir[hir[m.module].scope].is_empty() {
                    Some(m.name.clone())
                } else {
                    Some(format!("{}::", m.name))
                },
                command: Some(Command {
                    command: "editor.action.triggerSuggest".into(),
                    title: "Suggest".into(),
                    ..Default::default()
                }),
                ..CompletionItem::default()
            },
        )),
        _ => None,
    }
}

fn field_completion(
    doc: &Document,
    hir: &Hir,
    name: &str,
    ty: Type,
    existing_ident: Option<TextRange>,
) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        detail: Some(format!("{}", ty.fmt(hir))),
        // TODO: include field docs in types.
        documentation: None,
        kind: Some(CompletionItemKind::FIELD),
        insert_text: Some(name.to_string()),
        text_edit: existing_ident.map(|range| {
            CompletionTextEdit::Edit(TextEdit {
                new_text: name.to_string(),
                range: doc.mapper.range(range).unwrap().into_lsp(),
            })
        }),
        ..CompletionItem::default()
    }
}

fn trigger_completion() -> Command {
    Command {
        command: "editor.action.triggerSuggest".into(),
        title: "Suggest".into(),
        ..Default::default()
    }
}
