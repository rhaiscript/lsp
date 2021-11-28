#![allow(deprecated)]

use crate::{
    mapper::{LspExt, Mapper},
    util::signature_of,
};

use super::*;
use rhai_hir::{symbol::ObjectSymbol, Hir, Scope, Type};
use rhai_rowan::{
    ast::{AstNode, ExprFn, Rhai},
    syntax::{SyntaxElement, SyntaxKind},
};

pub(crate) async fn document_symbols(
    mut context: Context<World>,
    params: Params<DocumentSymbolParams>,
) -> Result<Option<DocumentSymbolResponse>, Error> {
    let p = params.required()?;

    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&p.text_document.uri) {
        Some(d) => d,
        None => return Ok(None),
    };

    let syntax = doc.parse.clone().into_syntax();

    let rhai = match Rhai::cast(syntax) {
        Some(r) => r,
        None => return Ok(None),
    };

    let source = match w.hir.source_for(&p.text_document.uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    // TODO: include the source only instead of the entire module.
    let root_scope = w.hir[w.hir[source].module].scope;

    Ok(Some(DocumentSymbolResponse::Nested(collect_symbols(
        &doc.mapper,
        &rhai,
        &w.hir,
        root_scope,
    ))))
}

fn collect_symbols(mapper: &Mapper, rhai: &Rhai, hir: &Hir, scope: Scope) -> Vec<DocumentSymbol> {
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
        let syntax = symbol_data
            .source
            .text_range
            .map(|range| rhai.syntax().covering_element(range))
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
                    kind: SymbolKind::Function,
                    name: ident.to_string(),
                    range: mapper
                        .range(expr.syntax().text_range())
                        .unwrap_or_default()
                        .into_lsp(),
                    selection_range: mapper
                        .range(ident.text_range())
                        .unwrap_or_default()
                        .into_lsp(),
                    detail: Some(signature_of(hir, rhai, symbol)),
                    children: Some(collect_symbols(mapper, rhai, hir, f.scope)),
                    tags: None,
                });
            }
            rhai_hir::symbol::SymbolKind::Block(block) => {
                document_symbols.extend(collect_symbols(mapper, rhai, hir, block.scope));
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
                        SymbolKind::Object
                    } else if decl.is_const {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
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
                                        rhai_hir::symbol::SymbolKind::Block(block) => {
                                            Some(collect_symbols(mapper, rhai, hir, block.scope))
                                        }
                                        _ => None,
                                    },
                                    None => None,
                                }
                            }
                            rhai_hir::symbol::SymbolKind::Object(object) => {
                                Some(collect_object_fields(mapper, rhai, hir, object))
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
    rhai: &Rhai,
    hir: &Hir,
    obj: &ObjectSymbol,
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
                kind: SymbolKind::Property,
                name: name.to_string(),
                range: mapper.range(range).unwrap_or_default().into_lsp(),
                selection_range: mapper.range(ident_range).unwrap_or_default().into_lsp(),
                detail: None,
                children: match field.value.map(|s| &hir[s]) {
                    Some(v) => match &v.kind {
                        rhai_hir::symbol::SymbolKind::Closure(closure) => {
                            match closure.expr.map(|s| &hir[s]) {
                                Some(exp) => match &exp.kind {
                                    rhai_hir::symbol::SymbolKind::Block(block) => {
                                        Some(collect_symbols(mapper, rhai, hir, block.scope))
                                    }
                                    _ => None,
                                },
                                None => None,
                            }
                        }
                        rhai_hir::symbol::SymbolKind::Object(object) => {
                            Some(collect_object_fields(mapper, rhai, hir, object))
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
