use rhai_hir::Module;
use rhai_rowan::{
    ast::{AstNode, Rhai},
    TextSize,
};

use crate::{
    mapper::{self, LspExt, Mapper},
    util::{documentation_for, signature_of},
};

use super::*;

pub(crate) async fn completion(
    mut context: Context<World>,
    params: Params<CompletionParams>,
) -> Result<Option<CompletionResponse>, Error> {
    let p = params.required()?;

    let uri = p.text_document_position.text_document.uri;
    let pos = p.text_document_position.position;

    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let offset = match doc.mapper.offset(mapper::Position::from_lsp(pos)) {
        Some(p) => p,
        None => return Ok(None),
    };

    let source = match w.hir.source_for(&uri) {
        Some(s) => s,
        None => return Ok(None),
    };
    let rhai = match Rhai::cast(doc.parse.clone_syntax()) {
        Some(r) => r,
        None => return Ok(None),
    };

    let mut completions = Vec::new();

    // Left side of the cursor.
    let search_offset = offset.checked_sub(1.into()).unwrap_or(offset);

    // add_visible_identifiers(
    //     &mut completions,
    //     &doc.mapper,
    //     module,
    //     &rhai,
    //     search_offset,
    //     offset,
    // );
    add_empty_object(&mut completions);

    completions.dedup_by(|a, b| a.label == b.label);

    Ok(Some(CompletionResponse::Array(completions)))
}

fn add_visible_identifiers(
    completions: &mut Vec<CompletionItem>,
    mapper: &Mapper,
    module: &Module,
    rhai: &Rhai,
    search_offset: TextSize,
    offset: TextSize,
) {
    // let reference_sym = module
    //     .symbol_at(offset, true)
    //     .and_then(|s| module[s].kind.as_reference().map(|r| (&module[s], r)));

    // completions.extend(
    //     module
    //         .visible_symbols_from_offset(search_offset)
    //         .filter_map(|symbol| {
    //             let symbol_data = &module[symbol];

    //             match &symbol_data.kind {
    //                 rhai_hir::symbol::SymbolKind::Fn(f) => Some(CompletionItem {
    //                     label: f.name.clone(),
    //                     detail: Some(signature_of(module, rhai, symbol)),
    //                     documentation: Some(Documentation::MarkupContent(MarkupContent {
    //                         kind: MarkupKind::Markdown,
    //                         value: documentation_for(module, rhai, symbol, false),
    //                     })),
    //                     kind: Some(CompletionItemKind::Function),
    //                     insert_text: Some(format!("{}($0)", &f.name)),
    //                     insert_text_format: Some(InsertTextFormat::Snippet),
    //                     ..CompletionItem::default()
    //                 }),
    //                 rhai_hir::symbol::SymbolKind::Decl(d) => Some(CompletionItem {
    //                     label: d.name.clone(),
    //                     detail: Some(signature_of(module, rhai, symbol)),
    //                     documentation: Some(Documentation::MarkupContent(MarkupContent {
    //                         kind: MarkupKind::Markdown,
    //                         value: documentation_for(module, rhai, symbol, false),
    //                     })),
    //                     kind: Some(if d.is_const {
    //                         CompletionItemKind::Constant
    //                     } else {
    //                         CompletionItemKind::Variable
    //                     }),
    //                     insert_text: Some(d.name.clone()),
    //                     ..CompletionItem::default()
    //                 }),
    //                 _ => None,
    //             }
    //             .map(|mut c| {
    //                 if let Some((r_data, r_sym)) = reference_sym {
    //                     if r_sym.name == c.label {
    //                         if let Some(range) = r_data
    //                             .text_range()
    //                             .and_then(|range| mapper.range(range).map(LspExt::into_lsp))
    //                         {
    //                             c.insert_text = None;
    //                             c.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
    //                                 range,
    //                                 new_text: c.label.clone(),
    //                             }));
    //                             c.insert_text_format = None;
    //                         }
    //                     }
    //                 }

    //                 c
    //             })
    //         }),
    // );
}

fn add_empty_object(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "#{ }".into(),
        detail: Some("new object".into()),
        kind: Some(CompletionItemKind::Snippet),
        insert_text: Some("#{ $0 }".into()),
        insert_text_format: Some(InsertTextFormat::Snippet),
        sort_text: Some("zzzz".into()),
        ..CompletionItem::default()
    });
}
