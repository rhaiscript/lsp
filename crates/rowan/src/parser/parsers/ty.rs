use super::Context;
use crate::parser::ParseErrorKind;
use crate::syntax::SyntaxKind::*;
use crate::T;

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_type(ctx: &mut Context) {
    parse_type_bp(ctx, 0);
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_bp(ctx: &mut Context, min_bp: u8) {
    ctx.start_node(TYPE);

    let ty_start = ctx.checkpoint();

    let token = require_token!(ctx in node);

    match token {
        T!["?"] => parse_type_unknown(ctx),
        T!["["] => parse_type_array(ctx),
        T!["("] => parse_type_tuple(ctx),
        T!["ident"] => parse_type_ident(ctx),
        T!["#{"] => parse_type_object(ctx),
        _ => {
            ctx.add_error(ParseErrorKind::UnexpectedToken);
            ctx.finish_node();
            return;
        }
    }

    // Currently only the `|` operator is allowed in types.
    while let Some(op @ T!["|"]) = ctx.token() {
        let (l_bp, r_bp) = match op.infix_binding_power() {
            Some(bp) => bp,
            None => {
                ctx.add_error(ParseErrorKind::UnexpectedToken);
                break;
            }
        };

        if l_bp < min_bp {
            break;
        }

        // Wrap the existing EXPR_SOMETHING into an EXPR for consistency.
        ctx.start_node_at(ty_start, TYPE);
        ctx.finish_node();

        ctx.eat();

        ctx.start_node_at(ty_start, TYPE_UNION);
        parse_type_bp(ctx, r_bp);
        ctx.finish_node();
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_array(ctx: &mut Context) {
    ctx.start_node(TYPE_ARRAY);

    let start_token = require_token!(ctx in node);

    match start_token {
        T!["["] => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedToken(T!["["]));
            ctx.finish_node();
            return;
        }
    }

    loop {
        let token = require_token!(ctx in node);
        if matches!((start_token, token), (T!["["], T!["]"])) {
            ctx.eat();
            break;
        }

        parse_type(ctx);

        let end_token = require_token!(ctx in node);

        match (start_token, end_token) {
            (T!["["], T!["]"]) => {
                ctx.eat();
                break;
            }
            (_, T![","]) => {
                ctx.eat();
            }
            _ => {
                ctx.add_error(ParseErrorKind::ExpectedToken(T![","]));
            }
        }
    }
    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_unknown(ctx: &mut Context) {
    ctx.start_node(TYPE_UNKNOWN);
    expect_token!(ctx in node, T!["?"]);
    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_tuple(ctx: &mut Context) {
    ctx.start_node(TYPE_TUPLE);

    let start_token = require_token!(ctx in node);

    match start_token {
        T!["("] => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedToken(T!["("]));
            ctx.finish_node();
            return;
        }
    }

    loop {
        let token = require_token!(ctx in node);
        if matches!((start_token, token), (T!["("], T![")"])) {
            ctx.eat();
            break;
        }

        parse_type(ctx);

        let end_token = require_token!(ctx in node);

        match (start_token, end_token) {
            (T!["("], T![")"]) => {
                ctx.eat();
                break;
            }
            (_, T![","]) => {
                ctx.eat();
            }
            _ => {
                ctx.add_error(ParseErrorKind::ExpectedToken(T![","]));
            }
        }
    }
    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_ident(ctx: &mut Context) {
    ctx.start_node(TYPE_IDENT);
    expect_token!(ctx in node, T!["ident"]);

    if let Some(T!["<"]) = ctx.token() {
        parse_type_generics(ctx);
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_generics(ctx: &mut Context) {
    ctx.start_node(TYPE_GENERICS);

    let start_token = require_token!(ctx in node);

    match start_token {
        T!["<"] => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedToken(T!["<"]));
            ctx.finish_node();
            return;
        }
    }

    loop {
        let token = require_token!(ctx in node);
        if matches!((start_token, token), (T!["<"], T![">"])) {
            ctx.eat();
            break;
        }

        parse_type(ctx);

        let end_token = require_token!(ctx in node);

        match (start_token, end_token) {
            (T!["<"], T![">"]) => {
                ctx.eat();
                break;
            }
            (_, T![","]) => {
                ctx.eat();
            }
            _ => {
                ctx.add_error(ParseErrorKind::ExpectedToken(T![","]));
            }
        }
    }
    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_object(ctx: &mut Context) {
    ctx.start_node(TYPE_OBJECT);
    expect_token!(ctx in node, T!["#{"]);

    let mut first = true;
    let mut separator = false;
    loop {
        let token = require_token!(ctx in node);

        if token == T!["}"] {
            ctx.eat();
            break;
        }

        if !first && !separator {
            ctx.add_error(ParseErrorKind::ExpectedToken(T![","]));
        }
        separator = false;

        parse_type_object_field(ctx);

        if let Some(T![","]) = ctx.token() {
            ctx.eat();
            separator = true;
        }

        first = false;
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_object_field(ctx: &mut Context) {
    ctx.start_node(TYPE_OBJECT_FIELD);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx in node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.start_node(DOC);
        ctx.eat();
        ctx.finish_node();
    }

    let token = require_token!(ctx in node);

    match token {
        T!["ident"] | T!["lit_str"] | T!["lit_int"] => {
            ctx.eat();
        }
        _ => {
            ctx.add_error(ParseErrorKind::UnexpectedToken);
            ctx.finish_node();
            return;
        }
    }

    expect_token!(ctx in node, T![":"]);

    parse_type(ctx);

    ctx.finish_node();
}
