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

    let mut parser = Parser::new(&p.text_document.text);
    parser.parse_file();
    let parse = parser.finish();

    let mapper = Mapper::new_utf16(&p.text_document.text, false);
    let uri = p.text_document.uri.clone();

    context
        .world()
        .lock()
        .unwrap()
        .documents
        .insert(p.text_document.uri, Document { parse, mapper });

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
    let mut parser = Parser::new(&change.text);
    parser.parse_file();
    let parse = parser.finish();

    let mapper = Mapper::new_utf16(&change.text, false);
    let uri = p.text_document.uri.clone();

    context
        .world()
        .lock()
        .unwrap()
        .documents
        .insert(p.text_document.uri, Document { parse, mapper });

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
