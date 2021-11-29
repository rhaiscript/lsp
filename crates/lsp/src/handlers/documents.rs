use super::*;

pub(crate) async fn document_change(
    context: Context<World>,
    params: Params<DidChangeTextDocumentParams>,
) {
    let mut p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    // We expect one full change
    let change = match p.content_changes.pop() {
        None => return,
        Some(c) => c,
    };

    crate::documents::update_document(context, p.text_document.uri, &change.text).await;
}

pub(crate) async fn document_open(
    context: Context<World>,
    params: Params<DidOpenTextDocumentParams>,
) {
    let p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    crate::documents::update_document(context, p.text_document.uri, &p.text_document.text).await;
}
