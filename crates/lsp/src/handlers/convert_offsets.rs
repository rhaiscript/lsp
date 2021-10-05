use crate::mapper::LspExt;

use super::*;

pub(crate) async fn convert_offsets(
    mut context: Context<World>,
    params: Params<ConvertOffsetsParams>,
) -> Result<Option<ConvertOffsetsResult>, Error> {
    let p = params.required()?;
    let w = context.world().lock().unwrap();
    let doc = match w.documents.get(&p.uri) {
        Some(d) => d,
        None => return Ok(None),
    };
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
