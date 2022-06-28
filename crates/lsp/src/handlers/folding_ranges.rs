use crate::{environment::Environment, world::World};

use az::SaturatingAs;
use lsp_async_stub::{
    rpc,
    util::{LspExt, Mapper},
    Context, Params,
};
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams, Range};
use rhai_rowan::syntax::{SyntaxElement, SyntaxKind::*, SyntaxNode};

pub(crate) async fn folding_ranges<E: Environment>(
    context: Context<World<E>>,
    params: Params<FoldingRangeParams>,
) -> Result<Option<Vec<FoldingRange>>, rpc::Error> {
    let p = params.required()?;

    let workspaces = context.workspaces.read().await;
    let ws = workspaces.by_document(&p.text_document.uri);

    let doc = ws.document(&p.text_document.uri)?;

    let syntax = doc.parse.clone().into_syntax();

    Ok(Some(
        syntax
            .descendants_with_tokens()
            .filter_map(|d| match d.kind() {
                EXPR_BLOCK | EXPR_OBJECT | COMMENT_BLOCK_DOC | COMMENT_BLOCK => {
                    doc.mapper.range(d.text_range()).map(|range| FoldingRange {
                        start_line: range.start.line.saturating_as(),
                        end_line: range.end.line.saturating_as(),
                        kind: match d.kind() {
                            COMMENT_BLOCK_DOC | COMMENT_BLOCK => Some(FoldingRangeKind::Comment),
                            _ => None,
                        },
                        ..FoldingRange::default()
                    })
                }
                _ => None,
            })
            .chain(collect_consecutive_comments(&doc.mapper, &syntax))
            .collect(),
    ))
}

fn collect_consecutive_comments(
    mapper: &Mapper,
    syntax: &SyntaxNode,
) -> impl Iterator<Item = FoldingRange> {
    let mut ranges = Vec::new();

    let mut last_comment_range: Option<Range> = None;
    let mut last_doc_range: Option<Range> = None;

    for token in syntax
        .descendants_with_tokens()
        .filter(|n| matches!(n.kind(), COMMENT_LINE | COMMENT_LINE_DOC))
        .filter_map(SyntaxElement::into_token)
    {
        match token.kind() {
            COMMENT_LINE => {
                let range = mapper
                    .range(token.text_range())
                    .unwrap_or_default()
                    .into_lsp();

                match last_comment_range {
                    Some(mut existing_range) => {
                        if range.end.line - existing_range.end.line > 1 {
                            if existing_range.end.line != existing_range.start.line {
                                ranges.push(FoldingRange {
                                    start_line: existing_range.start.line,
                                    end_line: existing_range.end.line,
                                    kind: Some(FoldingRangeKind::Comment),
                                    ..FoldingRange::default()
                                });
                            }

                            last_comment_range = Some(range);
                        } else {
                            existing_range.end = range.end;
                            last_comment_range = Some(existing_range);
                        }
                    }
                    None => last_comment_range = Some(range),
                }
            }
            COMMENT_LINE_DOC => {
                let range = mapper
                    .range(token.text_range())
                    .unwrap_or_default()
                    .into_lsp();

                match last_doc_range {
                    Some(mut existing_range) => {
                        if range.end.line - existing_range.end.line > 1 {
                            if existing_range.end.line != existing_range.start.line {
                                ranges.push(FoldingRange {
                                    start_line: existing_range.start.line,
                                    end_line: existing_range.end.line,
                                    kind: Some(FoldingRangeKind::Comment),
                                    ..FoldingRange::default()
                                });
                            }

                            last_doc_range = Some(range);
                        } else {
                            existing_range.end = range.end;
                            last_doc_range = Some(existing_range);
                        }
                    }
                    None => last_doc_range = Some(range),
                }
            }
            _ => unreachable!(),
        }
    }

    if let Some(existing_range) = last_comment_range {
        if existing_range.end.line != existing_range.start.line {
            ranges.push(FoldingRange {
                start_line: existing_range.start.line,
                end_line: existing_range.end.line,
                ..FoldingRange::default()
            });
        }
    }

    if let Some(existing_range) = last_doc_range {
        if existing_range.end.line != existing_range.start.line {
            ranges.push(FoldingRange {
                start_line: existing_range.start.line,
                end_line: existing_range.end.line,
                ..FoldingRange::default()
            });
        }
    }
    ranges.into_iter()
}
