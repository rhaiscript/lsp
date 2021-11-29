use super::*;
use crate::{mapper::LspExt, util::pluralize};

pub(crate) async fn code_lens(
    mut context: Context<World>,
    params: Params<CodeLensParams>,
) -> Result<Option<Vec<CodeLens>>, Error> {
    let p = params.required()?;

    let w = context.world().read();

    let doc = match w.documents.get(&p.text_document.uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let source = match w.hir.source_of(&p.text_document.uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    Ok(Some(
        w.hir
            .symbols()
            .filter(|(_, d)| d.source.is(source))
            .filter_map(|(sym, data)| {
                let r = match &data.kind {
                    rhai_hir::symbol::SymbolKind::Fn(d) => data
                        .selection_or_text_range()
                        .and_then(|range| doc.mapper.range(range).map(LspExt::into_lsp))
                        .map(|range| (&d.references, range)),
                    rhai_hir::symbol::SymbolKind::Decl(d) => {
                        if d.is_param || d.is_pat {
                            None
                        } else {
                            data.selection_or_text_range()
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

                let mut locations = Vec::new();
                collect_references(&w, sym, false, &mut locations);

                Some(CodeLens {
                    command: Some(Command {
                        title: format!(
                            "{} {}",
                            references.len(),
                            pluralize("reference", references.len())
                        ),
                        command: "editor.action.showReferences".into(),
                        arguments: Some(vec![
                            serde_json::to_value(p.text_document.uri.as_str()).unwrap(),
                            serde_json::to_value(&range.start).unwrap(),
                            serde_json::to_value(&locations).unwrap(),
                        ]),
                    }),
                    data: None,
                    range,
                })
            })
            .collect(),
    ))
}
