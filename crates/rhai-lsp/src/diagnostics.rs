use crate::world::{Document, World};
use lsp_async_stub::{util::LspExt, Context, RequestWriter};
use lsp_types::{
    notification, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location,
    PublishDiagnosticsParams, Url,
};
use rhai_common::{environment::Environment, util::Normalize};
use rhai_hir::{error::ErrorKind, Hir};

#[tracing::instrument(skip_all)]
pub(crate) async fn publish_all_diagnostics<E: Environment>(context: Context<World<E>>) {
    let workspaces = context.workspaces.read().await;
    let document_urls = workspaces
        .iter()
        .flat_map(|(_, ws)| ws.documents.keys().cloned())
        .collect::<Vec<_>>();
    drop(workspaces);

    for doc_url in document_urls {
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

    collect_hir_errors(&document_url.clone().normalize(), doc, &ws.hir, &mut diags);
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
        for error in hir.errors_for_source(source) {
            match &error.kind {
                ErrorKind::DuplicateFnParameter {
                    duplicate_symbol,
                    existing_symbol,
                } => diags.push(Diagnostic {
                    range: doc
                        .mapper
                        .range(
                            hir[*duplicate_symbol]
                                .selection_or_text_range()
                                .unwrap_or_default(),
                        )
                        .unwrap_or_default()
                        .into_lsp(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("Rhai".into()),
                    message: error.to_string(),
                    related_information: Some(Vec::from([DiagnosticRelatedInformation {
                        message: "parameter with the same name".into(),
                        location: Location {
                            range: doc
                                .mapper
                                .range(
                                    hir[*existing_symbol]
                                        .selection_or_text_range()
                                        .unwrap_or_default(),
                                )
                                .unwrap_or_default()
                                .into_lsp(),
                            uri: uri.clone(),
                        },
                    }])),
                    tags: None,
                    data: None,
                }),
                ErrorKind::UnresolvedReference {
                    reference_symbol,
                    similar_name: _,
                } => diags.push(Diagnostic {
                    range: doc
                        .mapper
                        .range(
                            hir[*reference_symbol]
                                .selection_or_text_range()
                                .unwrap_or_default(),
                        )
                        .unwrap_or_default()
                        .into_lsp(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("Rhai".into()),
                    message: error.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }),
                ErrorKind::UnresolvedImport { import } => diags.push(Diagnostic {
                    range: doc
                        .mapper
                        .range(hir[*import].selection_or_text_range().unwrap_or_default())
                        .unwrap_or_default()
                        .into_lsp(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("Rhai".into()),
                    message: error.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }),
                ErrorKind::NestedFunction { function } => diags.push(Diagnostic {
                    range: doc
                        .mapper
                        .range(hir[*function].selection_or_text_range().unwrap_or_default())
                        .unwrap_or_default()
                        .into_lsp(),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("Rhai".into()),
                    message: error.to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }),
            }
        }
    }
}
