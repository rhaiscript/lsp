use crate::watcher::WorkspaceWatcher;

use super::*;

pub(crate) async fn workspace_folders(
    mut context: Context<World>,
    params: Params<DidChangeWorkspaceFoldersParams>,
) {
    let p = match params.optional() {
        None => return,
        Some(p) => p,
    };

    let ctx = context.clone();

    let w = context.world().write();

    let watcher = match w
        .watcher
        .get_or_try_init(move || WorkspaceWatcher::new(ctx))
    {
        Ok(w) => w,
        Err(err) => {
            tracing::error!(error = %err, "failed to initialize workspace watcher");
            return;
        }
    };

    for added in p.event.added {
        if let Ok(p) = added.uri.to_file_path() {
            tracing::info!(path = ?p, "added workspace");
            watcher.add_workspace(&p);
        }
    }

    for removed in p.event.removed {
        if let Ok(p) = removed.uri.to_file_path() {
            tracing::info!(path = ?p, "removed workspace");
            watcher.remove_workspace(&p);
        }
    }
}
