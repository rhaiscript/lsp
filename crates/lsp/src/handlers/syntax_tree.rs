use super::*;

pub(crate) async fn syntax_tree(
    mut context: Context<World>,
    params: Params<SyntaxTreeParams>,
) -> Result<Option<SyntaxTreeResult>, Error> {
    let p = params.required()?;
    let w = context.world().read();
    let doc = match w.documents.get(&p.uri) {
        Some(d) => d,
        None => return Ok(None),
    };
    let syntax = doc.parse.clone().into_syntax();
    Ok(Some(SyntaxTreeResult {
        text: format!("{:#?}", &syntax),
        tree: serde_json::to_value(&syntax).unwrap_or_default(),
    }))
}
