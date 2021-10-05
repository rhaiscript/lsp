#![allow(deprecated)]

use crate::mapper::{LspExt, Mapper};

use super::*;
use rhai_rowan::ast::{AstNode, File, Stmt};

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

    let statements = match File::cast(syntax).map(|f| f.statements()) {
        Some(s) => s,
        _ => return Ok(None),
    };

    Ok(Some(DocumentSymbolResponse::Nested(collect_symbols(
        &doc.mapper,
        statements,
    ))))
}

fn collect_symbols(mapper: &Mapper, statements: impl Iterator<Item = Stmt>) -> Vec<DocumentSymbol> {
    let mut extra_symbols = Vec::new();

    let mut symbols: Vec<DocumentSymbol> = statements
        .filter_map(|stmt| {
            stmt.item()
                .and_then(|it| it.expr())
                .and_then(|expr| match expr {
                    rhai_rowan::ast::Expr::Let(e) => e.ident_token().map(|ident| DocumentSymbol {
                        deprecated: None,
                        kind: SymbolKind::Variable,
                        name: ident.to_string(),
                        range: mapper
                            .range(e.syntax().text_range())
                            .unwrap_or_default()
                            .into_lsp(),
                        selection_range: mapper
                            .range(ident.text_range())
                            .unwrap_or_default()
                            .into_lsp(),
                        detail: None,
                        children: None,
                        tags: None,
                    }),
                    rhai_rowan::ast::Expr::Const(e) => {
                        e.ident_token().map(|ident| DocumentSymbol {
                            deprecated: None,
                            kind: SymbolKind::Constant,
                            name: ident.to_string(),
                            range: mapper
                                .range(e.syntax().text_range())
                                .unwrap_or_default()
                                .into_lsp(),
                            selection_range: mapper
                                .range(ident.text_range())
                                .unwrap_or_default()
                                .into_lsp(),
                            detail: None,
                            children: None,
                            tags: None,
                        })
                    }
                    rhai_rowan::ast::Expr::Block(block) => {
                        extra_symbols.extend(collect_symbols(mapper, block.statements()));
                        None
                    }
                    rhai_rowan::ast::Expr::Fn(f) => f.ident_token().map(|ident| DocumentSymbol {
                        deprecated: None,
                        kind: SymbolKind::Function,
                        name: ident.to_string(),
                        range: mapper
                            .range(f.syntax().text_range())
                            .unwrap_or_default()
                            .into_lsp(),
                        selection_range: mapper
                            .range(ident.text_range())
                            .unwrap_or_default()
                            .into_lsp(),
                        detail: None,
                        children: f
                            .body()
                            .map(|body| collect_symbols(mapper, body.statements())),
                        tags: None,
                    }),
                    _ => None,
                })
        })
        .collect();

    symbols.extend(extra_symbols.into_iter());

    symbols
}
