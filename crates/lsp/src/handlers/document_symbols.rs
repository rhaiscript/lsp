#![allow(deprecated)]

use crate::{environment::Environment, utils::signature_of, world::World};
use lsp_async_stub::{
    rpc,
    util::{LspExt, Mapper},
    Context, Params,
};
use lsp_types::{DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind};
use rhai_hir::{source::Source, symbol::ObjectSymbol, Hir, Scope, Type};
use rhai_rowan::{
    ast::{AstNode, ExprFn},
    syntax::{SyntaxElement, SyntaxKind, SyntaxNode},
};

pub(crate) async fn document_symbols<E: Environment>(
    context: Context<World<E>>,
    params: Params<DocumentSymbolParams>,
) -> Result<Option<DocumentSymbolResponse>, rpc::Error> {
    let p = params.required()?;

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&p.text_document.uri);

    let doc = ws.document(&p.text_document.uri)?;

    let syntax = doc.parse.clone().into_syntax();

    let source = match ws.hir.source_of(&p.text_document.uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    let module = match ws.hir.module_by_url(&ws.hir[source].url) {
        Some(m) => m,
        None => return Ok(None),
    };

    let root_scope = ws.hir[module].scope;

    Ok(Some(DocumentSymbolResponse::Nested(collect_symbols(
        &doc.mapper,
        &syntax,
        &ws.hir,
        root_scope,
        source,
    ))))
}

fn collect_symbols(
    mapper: &Mapper,
    root: &SyntaxNode,
    hir: &Hir,
    scope: Scope,
    source: Source,
) -> Vec<DocumentSymbol> {
    let mut document_symbols = Vec::new();

    let scope_symbols = hir[scope]
        .symbols
        .iter()
        .map(|sym| (*sym, &hir[*sym]))
        .chain(
            hir[scope]
                .hoisted_symbols
                .iter()
                .map(|sym| (*sym, &hir[*sym])),
        );

    for (symbol, symbol_data) in scope_symbols {
        if !symbol_data.source.is(source) {
            continue;
        }

        let syntax = symbol_data
            .source
            .text_range
            .map(|range| root.covering_element(range))
            .and_then(SyntaxElement::into_node);

        match &symbol_data.kind {
            rhai_hir::symbol::SymbolKind::Fn(f) => {
                let expr = match syntax.and_then(ExprFn::cast) {
                    Some(e) => e,
                    None => continue,
                };

                let ident = match expr.ident_token() {
                    Some(token) => token,
                    None => continue,
                };

                document_symbols.push(DocumentSymbol {
                    deprecated: None,
                    kind: SymbolKind::FUNCTION,
                    name: ident.to_string(),
                    range: mapper
                        .range(expr.syntax().text_range())
                        .unwrap_or_default()
                        .into_lsp(),
                    selection_range: mapper
                        .range(ident.text_range())
                        .unwrap_or_default()
                        .into_lsp(),
                    detail: Some(signature_of(hir, root, symbol)),
                    children: Some(collect_symbols(mapper, root, hir, f.scope, source)),
                    tags: None,
                });
            }
            rhai_hir::symbol::SymbolKind::Block(block) => {
                document_symbols.extend(collect_symbols(mapper, root, hir, block.scope, source));
            }
            rhai_hir::symbol::SymbolKind::Decl(decl) => {
                let syntax = match syntax {
                    Some(s) => s,
                    None => continue,
                };

                let ident = syntax
                    .descendants_with_tokens()
                    .filter_map(SyntaxElement::into_token)
                    .find(|t| t.kind() == SyntaxKind::IDENT);

                let ident = match ident {
                    Some(id) => id,
                    None => continue,
                };

                document_symbols.push(DocumentSymbol {
                    deprecated: None,
                    kind: if matches!(&decl.ty, Type::Object(_)) {
                        SymbolKind::OBJECT
                    } else if decl.is_const {
                        SymbolKind::CONSTANT
                    } else {
                        SymbolKind::VARIABLE
                    },
                    name: ident.to_string(),
                    range: mapper
                        .range(syntax.text_range())
                        .unwrap_or_default()
                        .into_lsp(),
                    selection_range: mapper
                        .range(ident.text_range())
                        .unwrap_or_default()
                        .into_lsp(),
                    detail: None,
                    children: match decl
                        .value_scope
                        .map(|s| &hir[s])
                        .and_then(|s| s.symbols.first().map(|s| &hir[*s]))
                    {
                        Some(v) => match &v.kind {
                            rhai_hir::symbol::SymbolKind::Closure(closure) => {
                                match closure.expr.map(|s| &hir[s]) {
                                    Some(exp) => match &exp.kind {
                                        rhai_hir::symbol::SymbolKind::Block(block) => Some(
                                            collect_symbols(mapper, root, hir, block.scope, source),
                                        ),
                                        _ => None,
                                    },
                                    None => None,
                                }
                            }
                            rhai_hir::symbol::SymbolKind::Object(object) => {
                                Some(collect_object_fields(mapper, root, hir, object, source))
                            }
                            _ => None,
                        },
                        None => None,
                    },
                    tags: None,
                });
            }
            _ => {}
        }
    }

    document_symbols
}

fn collect_object_fields(
    mapper: &Mapper,
    root: &SyntaxNode,
    hir: &Hir,
    obj: &ObjectSymbol,
    source: Source,
) -> Vec<DocumentSymbol> {
    obj.fields
        .iter()
        .filter_map(|(name, field)| {
            let ident_range = match field.property_syntax.text_range {
                Some(r) => r,
                None => return None,
            };

            let range = match field.field_syntax.text_range {
                Some(r) => r,
                None => return None,
            };

            Some(DocumentSymbol {
                deprecated: None,
                kind: SymbolKind::PROPERTY,
                name: name.to_string(),
                range: mapper.range(range).unwrap_or_default().into_lsp(),
                selection_range: mapper.range(ident_range).unwrap_or_default().into_lsp(),
                detail: None,
                children: match field.value.map(|s| &hir[s]) {
                    Some(v) => match &v.kind {
                        rhai_hir::symbol::SymbolKind::Closure(closure) => {
                            match closure.expr.map(|s| &hir[s]) {
                                Some(exp) => match &exp.kind {
                                    rhai_hir::symbol::SymbolKind::Block(block) => Some(
                                        collect_symbols(mapper, root, hir, block.scope, source),
                                    ),
                                    _ => None,
                                },
                                None => None,
                            }
                        }
                        rhai_hir::symbol::SymbolKind::Object(object) => {
                            Some(collect_object_fields(mapper, root, hir, object, source))
                        }
                        _ => None,
                    },
                    None => None,
                },
                tags: None,
            })
        })
        .collect()
}
