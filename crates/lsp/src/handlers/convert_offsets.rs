use lsp_async_stub::{rpc, util::LspExt, Context, Params};

use crate::{
    environment::Environment,
    lsp_ext::request::{ConvertOffsetsParams, ConvertOffsetsResult},
    world::World,
};

pub(crate) async fn convert_offsets<E: Environment>(
    context: Context<World<E>>,
    params: Params<ConvertOffsetsParams>,
) -> Result<Option<ConvertOffsetsResult>, rpc::Error> {
    let p = params.required()?;
    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&p.uri);

    let doc = ws.document(&p.uri)?;

    Ok(Some(ConvertOffsetsResult {
        positions: p.positions.map(|offsets| {
            offsets
                .into_iter()
                .map(|offset| doc.mapper.position(offset).unwrap_or_default().into_lsp())
                .collect()
        }),
        ranges: p.ranges.map(|ranges| {
            ranges
                .into_iter()
                .map(|range| doc.mapper.range(range).unwrap_or_default().into_lsp())
                .collect()
        }),
    }))
}
