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
        workspaces
            .entry(added.uri.clone())
            .or_insert(Workspace::new(context.env.clone(), added.uri));
    }

    drop(workspaces);
    update_configuration(context).await;
}
