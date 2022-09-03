use std::sync::Arc;

use crate::world::World;
use lsp_async_stub::{rpc, util::LspExt, Context, Params};
use lsp_types::{DocumentFormattingParams, TextEdit};
use rhai_common::environment::Environment;

#[tracing::instrument(skip_all)]
pub(crate) async fn format<E: Environment>(
    context: Context<World<E>>,
    params: Params<DocumentFormattingParams>,
) -> Result<Option<Vec<TextEdit>>, rpc::Error> {
    let p = params.required()?;

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&p.text_document.uri);
    let doc = match ws.document(&p.text_document.uri) {
        Ok(d) => d,
        Err(error) => {
            tracing::debug!(%error, "failed to get document from workspace");
            return Ok(None);
        }
    };

    let format_opts = rhai_fmt::Options {
        indent_string: if p.options.insert_spaces {
            Arc::from(" ".repeat(p.options.tab_size as usize).as_str())
        } else {
            "\t".into()
        },
        ..Default::default()
    };

    Ok(Some(vec![TextEdit {
        range: doc.mapper.all_range().into_lsp(),
        new_text: rhai_fmt::format_syntax(doc.parse.clone_syntax(), format_opts),
    }]))
}
