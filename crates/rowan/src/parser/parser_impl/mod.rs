#![deny(unreachable_patterns)]

use super::context::Context;
use crate::parser::ParseErrorKind;
use crate::syntax::SyntaxKind::*;
use crate::T;

macro_rules! require_token {
    ($ctx:ident) => {
        match $ctx.token() {
            Some(t) => t,
            None => return $ctx.eat_error(ParseErrorKind::UnexpectedEof),
        }
    };

    ($ctx:ident, finish_node) => {
        match $ctx.token() {
            Some(t) => t,
            None => {
                $ctx.finish_node();
                $ctx.eat_error(ParseErrorKind::UnexpectedEof);
                return;
            }
        }
    };
}

macro_rules! expect_token {
    ($ctx:ident, finish_node, $($token:tt)*) => {
        match $ctx.token() {
            Some(t) => match t {
                $($token)* => {
                    $ctx.eat();
                },
                _ => {
                    $ctx.finish_node();
                    $ctx.eat_error(ParseErrorKind::ExpectedToken($($token)*));
                    return;
                }
            }
            None => {
                $ctx.finish_node();
                $ctx.eat_error(ParseErrorKind::ExpectedToken($($token)*));
                return;
            }
        }
    };
    ($ctx:ident, $($token:tt)+) => {
        match $ctx.token() {
            Some(t) => match t {
                $($token)* => {
                    $ctx.eat();
                },
                _ => return $ctx.eat_error(ParseErrorKind::UnexpectedEof),
            }
            None => return $ctx.eat_error(ParseErrorKind::UnexpectedEof),
        }
    };

}

impl<'src> super::Parser<'src> {
    pub fn parse_file(&mut self) {
        parse_file(&mut self.context);
    }
}

fn parse_file(ctx: &mut Context) {
    ctx.start_node(FILE);
    if let Some(SHEBANG) = ctx.token() {
        parse_shebang(ctx);
    }

    ctx.set_statement_closed(true);
    while ctx.token().is_some() {
        if !ctx.statement_closed() {
            ctx.add_error(ParseErrorKind::ExpectedToken(T![";"]));
        }

        parse_stmt(ctx);
    }
    ctx.set_statement_closed(true);

    ctx.finish_node()
}

fn parse_shebang(ctx: &mut Context) {
    let token = require_token!(ctx);

    if !matches!(token, SHEBANG) {
        return ctx.eat_error(ParseErrorKind::ExpectedToken(SHEBANG));
    }

    ctx.eat()
}

fn parse_stmt(ctx: &mut Context) {
    let token = require_token!(ctx);
    ctx.start_node(STMT);
    ctx.set_statement_closed(false);

    if token == T![";"] {
        ctx.finish_node();
        ctx.set_statement_closed(true);
        return ctx.eat();
    }

    parse_item(ctx);

    if let Some(token) = ctx.token() {
        if token == T![";"] {
            ctx.eat();
            ctx.set_statement_closed(true);
        }
    }

    ctx.finish_node();
}

fn parse_item(ctx: &mut Context) {
    ctx.start_node(ITEM);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx, finish_node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.eat();
    }

    parse_expr(ctx);

    ctx.finish_node();
}

fn parse_expr(ctx: &mut Context) {
    ctx.start_node(EXPR);
    let token = require_token!(ctx, finish_node);

    match token {
        T!["let"] => parse_expr_let(ctx),
        T!["const"] => parse_expr_const(ctx),
        T!["{"] => parse_expr_block(ctx),
        T!["fn"] => parse_expr_fn(ctx),
        LIT_INT | LIT_FLOAT | LIT_BOOL | LIT_STR | LIT_CHAR => parse_expr_lit(ctx),
        IDENT => parse_expr_ident(ctx),
        _ => ctx.eat_error(ParseErrorKind::UnexpectedToken),
    }

    ctx.finish_node();
}

fn parse_expr_ident(ctx: &mut Context) {
    ctx.start_node(EXPR_IDENT);
    parse_ident(ctx);
    ctx.finish_node();
}

fn parse_expr_lit(ctx: &mut Context) {
    ctx.start_node(EXPR_LIT);
    parse_lit(ctx);
    ctx.finish_node();
}

fn parse_expr_let(ctx: &mut Context) {
    ctx.start_node(EXPR_LET);

    expect_token!(ctx, finish_node, T!["let"]);
    expect_token!(ctx, finish_node, T!["ident"]);
    expect_token!(ctx, finish_node, T!["="]);

    parse_expr(ctx);

    ctx.finish_node();
}

fn parse_expr_const(ctx: &mut Context) {
    ctx.start_node(EXPR_CONST);

    expect_token!(ctx, finish_node, T!["const"]);
    expect_token!(ctx, finish_node, T!["ident"]);
    expect_token!(ctx, finish_node, T!["="]);

    parse_expr(ctx);

    ctx.finish_node();
}

fn parse_expr_block(ctx: &mut Context) {
    ctx.start_node(EXPR_BLOCK);

    expect_token!(ctx, finish_node, T!["{"]);

    ctx.set_statement_closed(true);
    loop {
        let token = require_token!(ctx, finish_node);

        if token == T!["}"] {
            break;
        }

        if !ctx.statement_closed() {
            ctx.add_error(ParseErrorKind::ExpectedToken(T![";"]));
        }

        parse_stmt(ctx);
    }

    expect_token!(ctx, finish_node, T!["}"]);

    // Blocks also don't require ";" when used as statements.
    ctx.set_statement_closed(true);

    ctx.finish_node();
}

fn parse_expr_fn(ctx: &mut Context) {
    ctx.start_node(EXPR_FN);

    expect_token!(ctx, finish_node, T!["fn"]);
    expect_token!(ctx, finish_node, T!["ident"]);

    parse_param_list(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

fn parse_param_list(ctx: &mut Context) {
    ctx.start_node(PARAM_LIST);

    let start_token = require_token!(ctx, finish_node);

    match start_token {
        T!["("] | T!["|"] => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![T!["("], T!["|"]]));
            return;
        }
    }

    loop {
        let token = require_token!(ctx, finish_node);
        if matches!(
            (start_token, token),
            (T!["("], T![")"]) | (T!["|"], T!["|"])
        ) {
            ctx.eat();
            break;
        }

        parse_param(ctx);

        let end_token = require_token!(ctx, finish_node);

        match (start_token, end_token) {
            (T!["("], T![")"]) | (T!["|"], T!["|"]) => {
                ctx.eat();
                break;
            }
            (_, T![","]) => {
                ctx.eat();
            }
            _ => {
                ctx.eat_error(ParseErrorKind::ExpectedToken(T![","]));
                break;
            }
        }
    }

    ctx.finish_node();
}

fn parse_param(ctx: &mut Context) {
    ctx.start_node(PARAM);

    expect_token!(ctx, finish_node, T!["ident"]);

    ctx.finish_node();
}

fn parse_lit(ctx: &mut Context) {
    ctx.start_node(LIT);
    let token = require_token!(ctx, finish_node);

    match token {
        LIT_INT | LIT_FLOAT | LIT_BOOL | LIT_STR | LIT_CHAR => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![
                LIT_INT, LIT_FLOAT, LIT_BOOL, LIT_STR, LIT_CHAR,
            ]));
        }
    }

    ctx.finish_node();
}

fn parse_ident(ctx: &mut Context) {
    let token = require_token!(ctx);

    match token {
        IDENT => ctx.eat(),
        _ => ctx.eat_error(ParseErrorKind::ExpectedToken(IDENT)),
    }
}

// Expr =
// | ExprUni
// | ExprBin
// | ExprParen
// | ExprArray
// | ExprIndex
// | ExprObject
// | ExprCall
// | ExprMethodCall
// | ExprAccess
// | ExprClosure
// | ExprIf
// | ExprLoop
// | ExprFor
// | ExprWhile
// | ExprBreak
// | ExprContinue
// | ExprSwitch
// | ExprReturn
// | ExprPath
// | ExprImport
