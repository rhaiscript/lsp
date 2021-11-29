#![allow(clippy::module_name_repetitions)]

use std::{ffi::OsStr, path::Path};

use lsp_async_stub::Context;
use lsp_types::Url;
use rhai_rowan::parser::Parser;
use walkdir::WalkDir;

use crate::{diagnostics, external::spawn, mapper::Mapper, Document, World};

pub(crate) async fn update_document(mut context: Context<World>, url: Url, text: &str) {
    let parser = Parser::new(text);
    let parse = if url.as_str().ends_with(".d.rhai") {
        parser.parse_def()
    } else {
        parser.parse()
    };

    let mapper = Mapper::new_utf16(text, false);

    let mut w = context.world().write();

    w.hir.add_source(&url, &parse.clone_syntax());
    w.hir.resolve_references();

    w.documents.insert(url.clone(), Document { parse, mapper });

    drop(w);

    spawn(diagnostics::publish_diagnostics(context.clone(), url));
}

pub(crate) async fn remove_document(mut context: Context<World>, url: Url) {
    let mut w = context.world().write();

    if let Some(source) = w.hir.source_of(&url) {
        w.hir.remove_source(source);
        w.hir.resolve_references();
    }

    drop(w);

    spawn(diagnostics::clear_diagnostics(context.clone(), url));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn collect_documents(context: Context<World>, root: &Path) {
    for entry in WalkDir::new(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                tracing::error!(error = %err, "failed to read file system");
                continue;
            }
        };

        if entry.path().extension() != Some(OsStr::new("rhai")) || !entry.file_type().is_file() {
            continue;
        }

        match format!("file://{}", entry.path().to_string_lossy()).parse::<Url>() {
            Ok(u) => {
                let context = context.clone();
                spawn(async move {
                    match tokio::fs::read_to_string(entry.path()).await {
                        Ok(text) => {
                            update_document(context, u, &text).await;
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "failed to read file");
                        }
                    }
                });
            }
            Err(err) => tracing::debug!(error = %err, "invalid file url"),
        }
    }
}
