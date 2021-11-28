#![allow(clippy::copy_iterator, clippy::from_over_into)]

use super::*;
use crate::mapper::{self, relative_position, LspExt, Mapper};
use enum_iterator::IntoEnumIterator;
use rhai_hir::symbol::ReferenceTarget;
use rhai_rowan::TextRange;
use std::cmp;

#[derive(Debug, Copy, Clone, IntoEnumIterator)]
#[repr(u32)]
pub(crate) enum SemanticTokenKind {
    Function,
    Variable,
}

impl Into<lsp_types::SemanticTokenType> for SemanticTokenKind {
    fn into(self) -> lsp_types::SemanticTokenType {
        match self {
            SemanticTokenKind::Function => lsp_types::SemanticTokenType::FUNCTION,
            SemanticTokenKind::Variable => lsp_types::SemanticTokenType::VARIABLE,
        }
    }
}

#[derive(Debug, Copy, Clone, IntoEnumIterator)]
#[repr(u32)]
pub(crate) enum SemanticTokenModifierKind {
    Readonly,
}

impl Into<lsp_types::SemanticTokenModifier> for SemanticTokenModifierKind {
    fn into(self) -> lsp_types::SemanticTokenModifier {
        match self {
            SemanticTokenModifierKind::Readonly => lsp_types::SemanticTokenModifier::READONLY,
        }
    }
}

pub(crate) async fn semantic_tokens(
    mut context: Context<World>,
    params: Params<SemanticTokensParams>,
) -> Result<Option<SemanticTokensResult>, Error> {
    let p = params.required()?;

    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&p.text_document.uri) {
        Some(d) => d,
        None => return Err(Error::new("document not found")),
    };

    let source = match w.hir.source_for(&p.text_document.uri) {
        Some(s) => s,
        None => return Ok(None),
    };
    let mut builder = SemanticTokensBuilder::new(&doc.mapper);

    for (_, symbol_data) in w.hir.symbols().filter(|(_, s)| s.source.is_part_of(source)) {
        match &symbol_data.kind {
            rhai_hir::symbol::SymbolKind::Fn(_) => {
                if let Some(range) = symbol_data.selection_range() {
                    builder.add_token(range, SemanticTokenKind::Function, &[]);
                }
            }
            rhai_hir::symbol::SymbolKind::Decl(d) => {
                if let Some(range) = symbol_data.selection_range() {
                    builder.add_token(
                        range,
                        SemanticTokenKind::Variable,
                        if d.is_const {
                            &[SemanticTokenModifierKind::Readonly]
                        } else {
                            &[]
                        },
                    );
                }
            }
            rhai_hir::symbol::SymbolKind::Reference(r) => {
                if let (Some(range), Some(ReferenceTarget::Symbol(target))) =
                    (symbol_data.selection_range(), &r.target)
                {
                    match &w.hir[*target].kind {
                        rhai_hir::symbol::SymbolKind::Fn(_) => {
                            builder.add_token(range, SemanticTokenKind::Function, &[]);
                        }
                        rhai_hir::symbol::SymbolKind::Decl(d) => builder.add_token(
                            range,
                            SemanticTokenKind::Variable,
                            if d.is_const {
                                &[SemanticTokenModifierKind::Readonly]
                            } else {
                                &[]
                            },
                        ),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let data = builder.finish();

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data,
    })))
}

struct SemanticTokensBuilder<'b> {
    tokens: Vec<(TextRange, SemanticTokenKind, Vec<SemanticTokenModifierKind>)>,
    mapper: &'b Mapper,
    last_position: Option<Position>,
}

impl<'b> SemanticTokensBuilder<'b> {
    fn new(mapper: &'b Mapper) -> Self {
        Self {
            tokens: Vec::new(),
            mapper,
            last_position: None,
        }
    }

    fn add_token(
        &mut self,
        range: TextRange,
        ty: SemanticTokenKind,
        modifiers: &[SemanticTokenModifierKind],
    ) {
        self.tokens.push((range, ty, modifiers.into()));
    }

    fn finish(mut self) -> Vec<SemanticToken> {
        self.tokens.sort_by(|a, b| {
            a.0.start()
                .partial_cmp(&b.0.start())
                .unwrap_or(cmp::Ordering::Equal)
        });

        let mut tokens = Vec::new();

        for (range, ty, modifiers) in self.tokens {
            let position = self.mapper.position(range.start()).unwrap();

            let relative = relative_position(
                position,
                mapper::Position::from_lsp(self.last_position.unwrap_or_default()),
            );

            tokens.push(SemanticToken {
                delta_line: relative.line as u32,
                delta_start: relative.character as u32,
                length: u32::from(range.end()) - u32::from(range.start()),
                token_type: ty as u32,
                token_modifiers_bitset: modifiers.iter().enumerate().fold(
                    0,
                    |mut total, (i, _)| {
                        total += 1 << i;
                        total
                    },
                ),
            });

            self.last_position = Some(position.into_lsp());
        }

        tokens
    }
}
