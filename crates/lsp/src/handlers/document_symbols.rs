#![allow(deprecated)]

use crate::mapper::LspExt;

use super::*;
use rhai_rowan::syntax::SyntaxKind::*;

pub(crate) async fn document_symbols(
    mut context: Context<World>,
    params: Params<DocumentSymbolParams>,
) -> Result<Option<DocumentSymbolResponse>, Error> {
    let p = params.required()?;

    let w = context.world().lock().unwrap();

    let doc = w
        .documents
        .get(&p.text_document.uri)
        .ok_or_else(Error::invalid_params)?;

    Ok(Some(DocumentSymbolResponse::Flat(
        doc.parse
            .clone()
            .into_syntax()
            .descendants()
            .filter_map(|n| match n.kind() {
                EXPR_FN => {
                    let name = match n.children_with_tokens().find(|t| t.kind() == IDENT) {
                        Some(t) => t.to_string(),
                        None => return None,
                    };

                    Some(SymbolInformation {
                        kind: SymbolKind::Function,
                        name,
                        container_name: None,
                        tags: None,
                        location: Location {
                            range: doc.mapper.range(n.text_range()).unwrap().into_lsp(),
                            uri: p.text_document.uri.clone(),
                        },
                        deprecated: None,
                    })
                }
                EXPR_CONST => {
                    let name = match n.children_with_tokens().find(|t| t.kind() == IDENT) {
                        Some(t) => t.to_string(),
                        None => return None,
                    };

                    Some(SymbolInformation {
                        kind: SymbolKind::Constant,
                        name,
                        container_name: None,
                        tags: None,
                        location: Location {
                            range: doc.mapper.range(n.text_range()).unwrap().into_lsp(),
                            uri: p.text_document.uri.clone(),
                        },
                        deprecated: None,
                    })
                }                
                EXPR_LET => {
                    let name = match n.children_with_tokens().find(|t| t.kind() == IDENT) {
                        Some(t) => t.to_string(),
                        None => return None,
                    };

                    Some(SymbolInformation {
                        kind: SymbolKind::Variable,
                        name,
                        container_name: None,
                        tags: None,
                        location: Location {
                            range: doc.mapper.range(n.text_range()).unwrap().into_lsp(),
                            uri: p.text_document.uri.clone(),
                        },
                        deprecated: None,
                    })
                }
                _ => None,
            })
            .collect(),
    )))
}
