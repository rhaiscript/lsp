use super::*;
use crate::{diagnostics, external::spawn, mapper::Mapper, Document};
use rhai_rowan::parser::Parser;

pub(crate) async fn document_open(
    mut context: Context<World>,
    params: Params<DidOpenTextDocumentParams>,
) {
    let p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    let parse = Parser::new(&p.text_document.text).parse();

    let mapper = Mapper::new_utf16(&p.text_document.text, false);
    let uri = p.text_document.uri.clone();

    let mut w = context.world().lock().unwrap();

    w.hir
        .add_module_from_syntax(uri.as_str(), &parse.clone_syntax());
    w.hir.resolve_references_in_module(uri.as_str());

    w.documents
        .insert(p.text_document.uri, Document { parse, mapper });

    drop(w);

    spawn(diagnostics::publish_diagnostics(context.clone(), uri));
}

pub(crate) async fn document_change(
    mut context: Context<World>,
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
    let parse = Parser::new(&change.text).parse();

    let mapper = Mapper::new_utf16(&change.text, false);
    let uri = p.text_document.uri.clone();

    let mut w = context.world().lock().unwrap();

    w.hir
        .add_module_from_syntax(uri.as_str(), &parse.clone_syntax());
    w.hir.resolve_references_in_module(uri.as_str());

    w.documents
        .insert(p.text_document.uri, Document { parse, mapper });

    drop(w);

    spawn(diagnostics::publish_diagnostics(context.clone(), uri));
}

pub(crate) async fn document_close(
    mut context: Context<World>,
    params: Params<DidCloseTextDocumentParams>,
) {
    let p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    context
        .world()
        .lock()
        .unwrap()
        .documents
        .remove(&p.text_document.uri);

    spawn(diagnostics::clear_diagnostics(context, p.text_document.uri));
}
