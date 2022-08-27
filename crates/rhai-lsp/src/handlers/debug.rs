use crate::{
    lsp_ext::request::{HirDumpParams, HirDumpResult},
    world::World,
};
use lsp_async_stub::{rpc, Context, Params};
use rhai_common::environment::Environment;
use rhai_hir::fmt::HirFmt;

pub(crate) async fn hir_dump<E: Environment>(
    context: Context<World<E>>,
    params: Params<HirDumpParams>,
) -> Result<Option<HirDumpResult>, rpc::Error> {
    let p = params.required()?;
    let workspaces = context.workspaces.read().await;
    let ws = if let Some(uri) = p.workspace_uri {
        match workspaces.get(&uri) {
            Some(w) => w,
            None => return Ok(None),
        }
    } else {
        workspaces.get_detached()
    };

    Ok(Some(HirDumpResult {
        hir: if ws.config.debug.hir.full {
            format!("{:#?}", ws.hir)
        } else {
            HirFmt::new(&ws.hir).with_source().to_string()
        },
    }))
}
