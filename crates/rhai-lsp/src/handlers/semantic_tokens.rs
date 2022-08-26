use crate::World;
use lsp_async_stub::{
    rpc::Error,
    util::{relative_range, LspExt, Mapper},
    Context, Params,
};
use lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensParams,
    SemanticTokensResult,
};
use rhai_common::environment::Environment;
use rhai_hir::{
    symbol::{BinaryOpKind, ReferenceTarget, SymbolKind},
    ty::Type,
    Hir, TypeKind,
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

    token_builder.extend(ws.hir.symbols().filter_map(|(symbol, data)| {
        if !data.source.is(source) {
            return None;
        }

        match &data.kind {
            SymbolKind::Decl(d) => {
                if let Some(ty) = token_for_ty(&ws.hir, ws.hir[symbol].ty) {
                    Some((ws.hir[symbol].selection_range()?, ty, vec![]))
                } else if d.is_const {
                    Some((
                        ws.hir[symbol].selection_range()?,
                        TokenType::Variable,
                        vec![TokenModifier::ReadOnly],
                    ))
                } else {
                    None
                }
            }
            SymbolKind::Ref(r) => {
                if let Some(&target_symbol) = r.target.as_ref().and_then(ReferenceTarget::as_symbol)
                {
                    if let Some(ty) = token_for_ty(&ws.hir, ws.hir[target_symbol].ty) {
                        Some((ws.hir[symbol].selection_range()?, ty, vec![]))
                    } else if let Some(d) = ws.hir[target_symbol].kind.as_decl() {
                        if  d.is_const {
                            Some((
                                ws.hir[symbol].selection_range()?,
                                TokenType::Variable,
                                vec![TokenModifier::ReadOnly],
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            SymbolKind::Path(p) => p.segments.last().and_then(|&sym| {
                if let Some(ty) = token_for_ty(&ws.hir, ws.hir[sym].ty) {
                    Some((ws.hir[sym].selection_range()?, ty, vec![]))
                } else if let Some(&target) = ws.hir[sym]
                    .kind
                    .as_reference()
                    .and_then(|r| r.target.as_ref().and_then(ReferenceTarget::as_symbol))
                {
                    if let Some(decl) = ws.hir[target].kind.as_decl() {
                        if decl.is_const {
                            Some((
                                ws.hir[sym].selection_range()?,
                                TokenType::Variable,
                                vec![TokenModifier::ReadOnly],
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            SymbolKind::Binary(b) => {
                b.op.as_ref()
                    .and_then(BinaryOpKind::as_custom)
                    .map(|c| (c.range, TokenType::CustomOperator, vec![]))
            }
            _ => None,
        }
    }));

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: token_builder.finish(),
    })))
}

fn token_for_ty(hir: &Hir, ty: Type) -> Option<TokenType> {
    match &hir[ty].kind {
        TypeKind::Module => Some(TokenType::Module),
        TypeKind::Fn(_) => Some(TokenType::Function),
        _ => None,
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum TokenType {
    CustomOperator,
    Function,
    Module,
    Variable,
}

impl TokenType {
    pub const LEGEND: &'static [SemanticTokenType] = &[
        SemanticTokenType::KEYWORD,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::NAMESPACE,
        SemanticTokenType::VARIABLE,
    ];
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
    tokens: Vec<(TextRange, TokenType, Vec<TokenModifier>)>,
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
        modifiers: impl IntoIterator<Item = TokenModifier>,
    ) {
        self.tokens
            .push((range, ty, modifiers.into_iter().collect()));
    }

    fn extend<M: IntoIterator<Item = TokenModifier>>(
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
                length: (relative
                    .end
                    .character
                    .saturating_sub(relative.start.character)) as u32,
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
