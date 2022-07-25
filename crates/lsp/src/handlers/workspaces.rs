use super::update_configuration;
use crate::{
    environment::Environment,
    world::{Workspace, World},
};
use lsp_async_stub::{Context, Params};
use lsp_types::DidChangeWorkspaceFoldersParams;

pub async fn workspace_change<E: Environment>(
    context: Context<World<E>>,
    params: Params<DidChangeWorkspaceFoldersParams>,
) {
    let p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    let mut workspaces = context.workspaces.write().await;

    for removed in p.event.removed {
        workspaces.remove(&removed.uri);
    }

    for added in p.event.added {
        let mut ws = Workspace::new(context.env.clone(), added.uri.clone());

        if let Err(error) = ws.load_rhai_config().await {
            tracing::error!(%error, "invalid configuration");
        }
        ws.load_all_files().await;

        workspaces.entry(added.uri).or_insert(ws);
    }

    drop(workspaces);
    update_configuration(context).await;
}
