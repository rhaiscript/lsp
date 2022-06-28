use crate::{
    environment::Environment,
    world::{Document, World},
};
use lsp_async_stub::{util::LspExt, Context, RequestWriter};
use lsp_types::{notification, Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Url};
use rhai_hir::Hir;

pub(crate) async fn publish_all_diagnostics<E: Environment>(context: Context<World<E>>) {
    let workspaces = context.workspaces.read().await;

    for doc_url in workspaces
        .iter()
        .flat_map(|(_, ws)| ws.documents.keys().cloned())
    {
        context
            .env
            .clone()
            .spawn_local(publish_diagnostics(context.clone(), doc_url));
    }
}

#[tracing::instrument(skip_all)]
pub(crate) async fn publish_diagnostics<E: Environment>(
    mut context: Context<World<E>>,
    document_url: Url,
) {
    let mut diags = Vec::new();

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&document_url);

    let doc = match ws.documents.get(&document_url) {
        Some(doc) => doc,
        None => return,
    };

    collect_syntax_errors(doc, &mut diags);
    drop(workspaces);

    context
        .write_notification::<notification::PublishDiagnostics, _>(Some(PublishDiagnosticsParams {
            uri: document_url.clone(),
            diagnostics: diags.clone(),
            version: None,
        }))
        .await
        .unwrap_or_else(|err| tracing::error!("{err}"));

    if !diags.is_empty() {
        return;
    }

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&document_url);

    let doc = match ws.documents.get(&document_url) {
        Some(doc) => doc,
        None => return,
    };

    collect_hir_errors(&document_url, doc, &ws.hir, &mut diags);
    drop(workspaces);

    context.clone().env.spawn_local(async move {
        context
            .write_notification::<notification::PublishDiagnostics, _>(Some(
                PublishDiagnosticsParams {
                    uri: document_url.clone(),
                    diagnostics: diags.clone(),
                    version: None,
                },
            ))
            .await
            .unwrap_or_else(|err| tracing::error!("{err}"));
    });
}

#[tracing::instrument(skip_all)]
pub(crate) async fn clear_diagnostics<E: Environment>(
    mut context: Context<World<E>>,
    document_url: Url,
) {
    context
        .write_notification::<notification::PublishDiagnostics, _>(Some(PublishDiagnosticsParams {
            uri: document_url,
            diagnostics: Vec::new(),
            version: None,
        }))
        .await
        .unwrap_or_else(|err| tracing::error!("{}", err));
}

#[tracing::instrument(skip_all)]
fn collect_syntax_errors(doc: &Document, diags: &mut Vec<Diagnostic>) {
    diags.extend(doc.parse.errors.iter().map(|e| {
        let range = doc.mapper.range(e.range).unwrap_or_default().into_lsp();
        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("Rhai".into()),
            message: e.kind.to_string(),
            related_information: None,
            tags: None,
            data: None,
        }
    }));
}

#[tracing::instrument(skip_all)]
fn collect_hir_errors(uri: &Url, doc: &Document, hir: &Hir, diags: &mut Vec<Diagnostic>) {
    if let Some(source) = hir.source_by_url(uri) {
        diags.extend(hir.errors_for_source(source).into_iter().map(|e| {
            let range = doc
                .mapper
                .range(e.text_range.unwrap_or_default())
                .unwrap_or_default()
                .into_lsp();
            Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("Rhai".into()),
                message: e.kind.to_string(),
                related_information: None,
                tags: None,
                data: None,
            }
        }));
    }
}
