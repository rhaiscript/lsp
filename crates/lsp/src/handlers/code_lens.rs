use super::*;
use crate::mapper::{LspExt, Mapper};
use rhai_hir::{Module, Symbol};

pub(crate) async fn code_lens(
    mut context: Context<World>,
    params: Params<CodeLensParams>,
) -> Result<Option<Vec<CodeLens>>, Error> {
    let p = params.required()?;

    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&p.text_document.uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let module = match w.hir.get_module(p.text_document.uri.as_str()) {
        Some(m) => m,
        None => return Ok(None),
    };

    Ok(Some(
        module
            .symbols()
            .filter_map(|(_, data)| {
                let r = match &data.kind {
                    rhai_hir::symbol::SymbolKind::Fn(d) => data
                        .selection_syntax
                        .and_then(|s| s.text_range)
                        .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
                        .map(|range| (&d.references, range)),
                    rhai_hir::symbol::SymbolKind::Decl(d) => {
                        if d.is_param || d.is_pat {
                            None
                        } else {
                            data.selection_syntax
                                .and_then(|s| s.text_range)
                                .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
                                .map(|range| (&d.references, range))
                        }
                    }
                    _ => None,
                };

                let (references, range) = match r {
                    Some(r) => r,
                    None => return None,
                };

                Some(CodeLens {
                    command: Some(Command {
                        title: format!("{} references", references.len()),
                        command: "editor.action.showReferences".into(),
                        arguments: Some(vec![
                            serde_json::to_value(p.text_document.uri.as_str()).unwrap(),
                            serde_json::to_value(&range.start).unwrap(),
                            serde_json::to_value(&collect_locations(
                                module,
                                references.iter().copied(),
                                &doc.mapper,
                                &p.text_document.uri,
                            ))
                            .unwrap(),
                        ]),
                    }),
                    data: None,
                    range,
                })
            })
            .collect(),
    ))
}

fn collect_locations(
    module: &Module,
    symbols: impl Iterator<Item = Symbol>,
    mapper: &Mapper,
    uri: &Url,
) -> Vec<Location> {
    symbols
        .filter_map(|symbol| {
            module[symbol]
                .syntax
                .and_then(|syntax| syntax.text_range)
                .and_then(|range| mapper.range(range).map(LspExt::into_lsp))
        })
        .map(|range: Range| Location {
            uri: uri.clone(),
            range,
        })
        .collect::<Vec<Location>>()
}
