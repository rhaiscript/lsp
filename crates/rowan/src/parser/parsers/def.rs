use super::Context;
use crate::parser::parsers::parse_expr;
use crate::parser::{Parse, ParseErrorKind, Parser};
use crate::syntax::SyntaxKind::*;
use crate::T;

impl<'src> Parser<'src> {
    /// Parse Rhai definition code with [`parse_rhai_def`], and finish the parser.
    pub fn parse_def(mut self) -> Parse {
        self.execute(parse_rhai_def);
        self.finish()
    }
}

/// Parse the beginning of a rhai definition file.
///
/// It is used to determine if a given source should
/// be parsed as a definition file.
pub(crate) fn parse_def_header(ctx: &mut Context) {
    ctx.start_node(DEF_MODULE_DECL);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx in node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.start_node(DOC);
        ctx.eat();
        ctx.finish_node();
    }

    expect_token!(ctx in node, T!["module"]);

    ctx.finish_node();
}

/// Parse a Rhai definition file.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_rhai_def(ctx: &mut Context) {
    ctx.start_node(RHAI_DEF);

    parse_def_module_decl(ctx);

    ctx.set_statement_closed(true);
    while ctx.token().is_some() {
        if !ctx.statement_closed() {
            ctx.add_error(ParseErrorKind::ExpectedToken(T![";"]));
        }

        parse_def_stmt(ctx);
    }
    ctx.set_statement_closed(true);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_module_decl(ctx: &mut Context) {
    ctx.start_node(DEF_MODULE_DECL);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx in node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.start_node(DOC);
        ctx.eat();
        ctx.finish_node();
    }

    parse_def_module(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_stmt(ctx: &mut Context) {
    ctx.start_node(DEF_STMT);
    let token = require_token!(ctx in node);
    ctx.set_statement_closed(false);

    if token == T![";"] {
        ctx.finish_node();
        ctx.set_statement_closed(true);
        return ctx.eat();
    }

    parse_def_item(ctx);

    if let Some(token) = ctx.token() {
        if token == T![";"] {
            ctx.eat();
            ctx.set_statement_closed(true);
        }
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_item(ctx: &mut Context) {
    ctx.start_node(DEF_ITEM);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx in node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.start_node(DOC);
        ctx.eat();
        ctx.finish_node();
    }

    parse_def(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def(ctx: &mut Context) {
    ctx.start_node(DEF);

    let token = require_token!(ctx in node);

    match token {
        T!["import"] => parse_def_import(ctx),
        T!["const"] => parse_def_const(ctx),
        T!["let"] => parse_def_let(ctx),
        T!["fn"] => parse_def_fn(ctx),
        T!["ident"] if ctx.slice() == "op" => parse_def_op(ctx),
        T!["ident"] if ctx.slice() == "type" => parse_def_type(ctx),
        _ => {
            ctx.add_error(ParseErrorKind::UnexpectedToken);
        }
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_module(ctx: &mut Context) {
    ctx.start_node(DEF_MODULE);

    expect_token!(ctx in node, T!["module"]);

    match ctx.token() {
        Some(T!["ident"] | T!["lit_str"] | T!["static"]) => {
            ctx.eat();
        }
        _ => {}
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_import(ctx: &mut Context) {
    ctx.start_node(DEF_IMPORT);

    expect_token!(ctx in node, T!["import"]);
    parse_expr(ctx);

    if matches!(ctx.token(), Some(T!["as"])) {
        ctx.eat();
        expect_token!(ctx in node, T!["ident"]);
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_const(ctx: &mut Context) {
    ctx.start_node(DEF_CONST);

    expect_token!(ctx in node, T!["const"]);
    expect_token!(ctx in node, T!["ident"]);

    if let Some(T![":"]) = ctx.token() {
        ctx.eat();
        super::ty::parse_type(ctx);
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_let(ctx: &mut Context) {
    ctx.start_node(DEF_LET);

    expect_token!(ctx in node, T!["let"]);
    expect_token!(ctx in node, T!["ident"]);

    if let Some(T![":"]) = ctx.token() {
        ctx.eat();
        super::ty::parse_type(ctx);
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_fn(ctx: &mut Context) {
    ctx.start_node(DEF_FN);

    expect_token!(ctx in node, T!["fn"]);

    let token = require_token!(ctx in node);

    if token == T!["ident"] && ctx.slice() == "get" || ctx.slice() == "set" {
        ctx.eat();

        if ctx.token() == Some(T!["ident"]) {
            ctx.eat();
        }
    } else {
        expect_token!(ctx in node, T!["ident"]);
    }

    if !matches!(ctx.token(), Some(T!["("])) {
        ctx.add_error(ParseErrorKind::ExpectedToken(T!["("]));
        ctx.finish_node();
        return;
    }

    parse_typed_param_list(ctx);

    if let Some(T!["->"]) = ctx.token() {
        ctx.eat();
        super::ty::parse_type(ctx);
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_op(ctx: &mut Context) {
    ctx.start_node(DEF_OP);

    let token = require_token!(ctx in node);

    if !matches!(token, T!["ident"]) || ctx.slice() != "op" {
        ctx.add_error(ParseErrorKind::UnexpectedToken);
        ctx.finish_node();
        return;
    }
    ctx.eat();

    let token = require_token!(ctx in node);

    if token.infix_binding_power().is_some()
        || token.prefix_binding_power().is_some()
        || matches!(token, T!["ident"])
    {
        ctx.eat_as(T!["ident"]);
    } else {
        ctx.add_error(ParseErrorKind::UnexpectedToken);
        ctx.finish_node();
        return;
    }

    if !matches!(ctx.token(), Some(T!["("])) {
        ctx.add_error(ParseErrorKind::ExpectedToken(T!["("]));
        ctx.finish_node();
        return;
    }

    parse_type_list(ctx);

    if let Some(T!["->"]) = ctx.token() {
        ctx.eat();
        super::ty::parse_type(ctx);
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_def_type(ctx: &mut Context) {
    ctx.start_node(DEF_TYPE);

    let token = require_token!(ctx in node);

    if !matches!(token, T!["ident"]) || ctx.slice() != "type" {
        ctx.add_error(ParseErrorKind::UnexpectedToken);
        ctx.finish_node();
        return;
    }
    ctx.eat();

    expect_token!(ctx in node, T!["ident"]);
    expect_token!(ctx in node, T!["="]);

    super::ty::parse_type(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_typed_param_list(ctx: &mut Context) {
    ctx.start_node(TYPED_PARAM_LIST);

    let start_token = require_token!(ctx in node);

    match start_token {
        T!["||"] => {
            // special case
            ctx.discard();
            ctx.insert_token(T!["|"], "|");
            ctx.insert_token(T!["|"], "|");
            ctx.finish_node();
            return;
        }
        T!["("] | T!["|"] => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![T!["("], T!["|"]]));
            ctx.finish_node();
            return;
        }
    }

    loop {
        let token = require_token!(ctx in node);
        if matches!(
            (start_token, token),
            (T!["("], T![")"]) | (T!["|"], T!["|"])
        ) {
            ctx.eat();
            break;
        }

        parse_typed_param(ctx);

        let end_token = require_token!(ctx in node);

        match (start_token, end_token) {
            (T!["("], T![")"]) | (T!["|"], T!["|"]) => {
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
fn parse_typed_param(ctx: &mut Context) {
    ctx.start_node(TYPED_PARAM);

    let token = require_token!(ctx in node);

    if let T!["..."] = token {
        ctx.eat();
    }

    if !matches!(ctx.token(), Some(T!["_"] | T!["ident"])) {
        ctx.add_error(ParseErrorKind::ExpectedOneOfTokens(vec![
            T!["_"],
            T!["ident"],
        ]));
        ctx.finish_node();
        return;
    }

    ctx.eat_as(T!["ident"]);

    expect_token!(ctx in node, T![":"]);

    super::ty::parse_type(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_type_list(ctx: &mut Context) {
    ctx.start_node(TYPE_LIST);

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

        super::ty::parse_type(ctx);

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
