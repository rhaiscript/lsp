use lsp_async_stub::{rpc, Context, Params};

use crate::{
    environment::Environment,
    lsp_ext::request::{SyntaxTreeParams, SyntaxTreeResult},
    world::World,
};

pub(crate) async fn syntax_tree<E: Environment>(
    context: Context<World<E>>,
    params: Params<SyntaxTreeParams>,
) -> Result<Option<SyntaxTreeResult>, rpc::Error> {
    let p = params.required()?;
    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&p.uri);

    let doc = ws.document(&p.uri)?;

    let syntax = doc.parse.clone().into_syntax();
    Ok(Some(SyntaxTreeResult {
        text: format!("{:#?}", &syntax),
        tree: serde_json::to_value(&syntax).unwrap_or_default(),
    }))
}
