#![allow(clippy::module_name_repetitions)]

use lsp_async_stub::{Context, RequestWriter};
use lsp_types::{notification, Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Url};
use rhai_hir::Module;
use rhai_rowan::parser::Parse;
use tracing::error;

use crate::{
    external::spawn,
    mapper::{LspExt, Mapper},
    World,
};

pub async fn publish_diagnostics(mut context: Context<World>, uri: Url) {
    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&uri) {
        Some(d) => d.clone(),
        None => {
            // Doesn't exist anymore
            return;
        }
    };

    let mut diagnostics = Vec::new();

    syntax_diagnostics(&mut diagnostics, &doc.parse, &doc.mapper);

    // if let Some(m) = w.hir.get_module(uri.as_str()) {
    //     hir_diagnostics(&mut diagnostics, m, &doc.mapper);
    // };

    drop(w);

    // FIXME: why is another `spawn` required here?
    //  the compiler complains about the future not being Sync otherwise,
    //  but I don't see why.
    spawn(async move {
        context
            .write_notification::<notification::PublishDiagnostics, _>(Some(
                PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics,
                    version: None,
                },
            ))
            .await
            .unwrap_or_else(|error| error!(%error));
    });
}

pub async fn clear_diagnostics(mut context: Context<World>, uri: Url) {
    context
        .write_notification::<notification::PublishDiagnostics, _>(Some(PublishDiagnosticsParams {
            uri,
            diagnostics: Vec::new(),
            version: None,
        }))
        .await
        .unwrap_or_else(|error| error!(%error));
}

fn syntax_diagnostics(diagnostics: &mut Vec<Diagnostic>, parse: &Parse, mapper: &Mapper) {
    diagnostics.extend(parse.errors.iter().map(|e| {
        let range = mapper.range(e.range).unwrap_or_default().into_lsp();
        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::Error),
            code: None,
            code_description: None,
            source: Some("Rhai".into()),
            message: format!("{}", e),
            related_information: None,
            tags: None,
            data: None,
        }
    }));
}

fn hir_diagnostics(diagnostics: &mut Vec<Diagnostic>, module: &Module, mapper: &Mapper) {
    // diagnostics.extend(
    //     module
    //         .collect_errors()
    //         .iter()
    //         .filter(|e| e.text_range.is_some())
    //         .map(|e| {
    //             let range = mapper
    //                 .range(e.text_range.unwrap())
    //                 .unwrap_or_default()
    //                 .into_lsp();
    //             Diagnostic {
    //                 range,
    //                 severity: Some(DiagnosticSeverity::Error),
    //                 code: None,
    //                 code_description: None,
    //                 source: Some("Rhai".into()),
    //                 message: format!("{}", e),
    //                 related_information: None,
    //                 tags: None,
    //                 data: None,
    //             }
    //         }),
    // );
}
