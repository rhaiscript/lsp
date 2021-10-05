use crate::mapper::{LspExt, Mapper};

use super::*;
use rhai_rowan::syntax::{SyntaxKind::*, SyntaxNode};

pub(crate) async fn folding_ranges(
    mut context: Context<World>,
    params: Params<FoldingRangeParams>,
) -> Result<Option<Vec<FoldingRange>>, Error> {
    let p = params.required()?;

    let w = context.world().lock().unwrap();

    let doc = match w.documents.get(&p.text_document.uri) {
        Some(d) => d,
        None => return Ok(None),
    };

    let syntax = doc.parse.clone().into_syntax();

    Ok(Some(
        syntax
            .descendants_with_tokens()
            .filter_map(|d| match d.kind() {
                EXPR_BLOCK | EXPR_OBJECT | COMMENT_BLOCK_DOC | COMMENT_BLOCK => {
                    doc.mapper.range(d.text_range()).map(|range| FoldingRange {
                        start_line: range.start.line as u32,
                        end_line: range.end.line as u32,
                        kind: match d.kind() {
                            COMMENT_BLOCK_DOC | COMMENT_BLOCK => Some(FoldingRangeKind::Comment),
                            _ => None,
                        },
                        ..Default::default()
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
        .filter_map(|e| e.into_token())
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
                                    ..Default::default()
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
                                    ..Default::default()
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
                ..Default::default()
            });
        }
    }

    if let Some(existing_range) = last_doc_range {
        if existing_range.end.line != existing_range.start.line {
            ranges.push(FoldingRange {
                start_line: existing_range.start.line,
                end_line: existing_range.end.line,
                ..Default::default()
            });
        }
    }
    ranges.into_iter()
}
