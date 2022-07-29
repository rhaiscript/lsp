use crate::{environment::Environment, World};
use lsp_async_stub::{
    rpc::Error,
    util::{relative_range, LspExt, Mapper},
    Context, Params,
};
use lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensParams,
    SemanticTokensResult,
};
use rhai_rowan::TextRange;

#[tracing::instrument(skip_all)]
pub(crate) async fn semantic_tokens<E: Environment>(
    context: Context<World<E>>,
    params: Params<SemanticTokensParams>,
) -> Result<Option<SemanticTokensResult>, Error> {
    let p = params.required()?;

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&p.text_document.uri);

    let doc = ws.document(&p.text_document.uri)?;

    if !ws.config.syntax.semantic_tokens {
        return Ok(None);
    }

    let source = match ws.hir.source_by_url(&p.text_document.uri) {
        Some(s) => s,
        None => return Ok(None),
    };

    let mut token_builder = SemanticTokensBuilder::new(&doc.mapper);

    token_builder.extend(ws.hir.symbols().filter_map(|(_, data)| {
        if !data.source.is(source) {
            return None;
        }

        Some((
            data.kind.as_binary()?.op.as_ref()?.as_custom()?.range,
            TokenType::CustomOperator,
            [],
        ))
    }));

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: token_builder.finish(),
    })))
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum TokenType {
    CustomOperator,
}

impl TokenType {
    pub const LEGEND: &'static [SemanticTokenType] = &[SemanticTokenType::KEYWORD];
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum TokenModifier {
    ReadOnly,
}

impl TokenModifier {
    pub const MODIFIERS: &'static [SemanticTokenModifier] = &[SemanticTokenModifier::READONLY];
}

struct SemanticTokensBuilder<'b> {
    tokens: Vec<(TextRange, TokenType, Vec<SemanticTokenModifier>)>,
    // tokens: Vec<SemanticToken>,
    mapper: &'b Mapper,
}

impl<'b> SemanticTokensBuilder<'b> {
    fn new(mapper: &'b Mapper) -> Self {
        Self {
            tokens: Vec::new(),
            mapper,
        }
    }

    fn add(
        &mut self,
        range: TextRange,
        ty: TokenType,
        modifiers: impl IntoIterator<Item = SemanticTokenModifier>,
    ) {
        self.tokens
            .push((range, ty, modifiers.into_iter().collect()));
    }

    fn extend<M: IntoIterator<Item = SemanticTokenModifier>>(
        &mut self,
        iter: impl IntoIterator<Item = (TextRange, TokenType, M)>,
    ) {
        for (range, ty, modifiers) in iter {
            self.add(range, ty, modifiers);
        }
    }

    fn finish(mut self) -> Vec<SemanticToken> {
        self.tokens.sort_by_key(|(range, ..)| range.start());

        let mut last_range = None;

        let mut tokens = Vec::with_capacity(self.tokens.len());

        for (range, ty, modifiers) in self.tokens {
            let range = self.mapper.range(range).unwrap();

            let relative = relative_range(
                range,
                lsp_async_stub::util::Range::from_lsp(last_range.unwrap_or_default()),
            );

            #[allow(clippy::cast_possible_truncation)]
            tokens.push(SemanticToken {
                delta_line: relative.start.line as u32,
                delta_start: relative.start.character as u32,
                length: (relative.end.character.saturating_sub(relative.start.character)) as u32,
                token_type: ty as u32,
                token_modifiers_bitset: modifiers.iter().enumerate().fold(
                    0,
                    |mut total, (i, _)| {
                        total += 1 << i;
                        total
                    },
                ),
            });

            last_range = Some(range.into_lsp());
        }

        tokens
    }
}
