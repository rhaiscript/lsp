#![deny(unreachable_patterns)]

use super::context::Context;
use crate::parser::ParseErrorKind;
use crate::syntax::{SyntaxKind, SyntaxKind::*};
use crate::T;

use tracing::instrument;

macro_rules! require_token {
    ($ctx:ident) => {
        match $ctx.token() {
            Some(t) => t,
            None => return $ctx.eat_error(ParseErrorKind::UnexpectedEof),
        }
    };

    ($ctx:ident in node) => {
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
    ($ctx:ident in node, $($token:tt)*) => {
        match $ctx.token() {
            Some(t) => match t {
                $($token)* => {
                    $ctx.eat();
                },
                _ => {
                    $ctx.finish_node();
                    $ctx.add_error(ParseErrorKind::ExpectedToken($($token)*));
                    return;
                }
            }
            None => {
                $ctx.finish_node();
                $ctx.add_error(ParseErrorKind::ExpectedToken($($token)*));
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
                _ => return $ctx.add_error(ParseErrorKind::UnexpectedEof),
            }
            None => return $ctx.add_error(ParseErrorKind::UnexpectedEof),
        }
    };
}

macro_rules! expect_token_eat_error {
    ($ctx:ident in node, $($token:tt)*) => {
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
    pub fn parse(mut self) -> super::Parse {
        self.execute(parse_file);
        self.finish()
    }
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_file(ctx: &mut Context) {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_shebang(ctx: &mut Context) {
    let token = require_token!(ctx);

    if !matches!(token, SHEBANG) {
        return ctx.eat_error(ParseErrorKind::ExpectedToken(SHEBANG));
    }

    ctx.eat()
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_stmt(ctx: &mut Context) {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_item(ctx: &mut Context) {
    ctx.start_node(ITEM);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx in node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.eat();
    }

    parse_expr(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr(ctx: &mut Context) {
    parse_expr_bp(ctx, 0);
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_bp(ctx: &mut Context, min_bp: u8) {
    ctx.start_node(EXPR);

    let expr_start = ctx.checkpoint();

    let token = require_token!(ctx in node);

    match token {
        T!["let"] => {
            parse_expr_let(ctx);
            ctx.finish_node();
            return;
        }
        T!["const"] => {
            parse_expr_const(ctx);
            ctx.finish_node();
            return;
        }
        T!["#{"] => {
            parse_expr_object(ctx);
            if let Some(t) = ctx.token() {
                if t.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["fn"] => {
            parse_expr_fn(ctx);
            ctx.finish_node();
            return;
        }
        T!["|"] | T!["||"] => {
            // boolean "or" is a special case unfortunately.
            parse_expr_closure(ctx);
            ctx.finish_node();
            return;
        }
        T!["if"] => {
            parse_expr_if(ctx);

            if let Some(t) = ctx.token() {
                if t.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["loop"] => {
            parse_expr_loop(ctx);
            if let Some(t) = ctx.token() {
                if t.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["for"] => {
            parse_expr_for(ctx);
            ctx.finish_node();
            return;
        }
        T!["while"] => {
            parse_expr_while(ctx);
            ctx.finish_node();
            return;
        }
        T!["break"] => {
            parse_expr_break(ctx);
            ctx.finish_node();
            return;
        }
        T!["continue"] => {
            parse_expr_continue(ctx);
            ctx.finish_node();
            return;
        }
        T!["return"] => {
            parse_expr_return(ctx);
            ctx.finish_node();
            return;
        }
        T!["switch"] => {
            parse_expr_switch(ctx);
            if let Some(t) = ctx.token() {
                if t.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["import"] => {
            parse_expr_import(ctx);
            ctx.finish_node();
            return;
        }
        T!["export"] => {
            parse_expr_export(ctx);
            ctx.finish_node();
            return;
        }
        T!["{"] => parse_expr_block(ctx),
        T!["("] => parse_expr_paren(ctx),
        T!["["] => parse_expr_array(ctx),
        LIT_INT | LIT_FLOAT | LIT_BOOL | LIT_STR | LIT_CHAR => parse_expr_lit(ctx),
        IDENT => parse_expr_path_or_ident(ctx),
        op => {
            if let Some(r_bp) = op.prefix_binding_power() {
                ctx.start_node_at(expr_start, EXPR_UNARY);
                ctx.eat();
                parse_expr_bp(ctx, r_bp);
                ctx.finish_node(); // EXPR_UNARY
            } else {
                ctx.eat_error(ParseErrorKind::UnexpectedToken);
            }

            ctx.finish_node(); // EXPR
            return;
        }
    }

    loop {
        let op = match ctx.token() {
            Some(T![";"] | T![","] | T!["{"] | T!["}"] | T![")"] | T!["]"] | T!["=>"]) | None => {
                break
            }
            Some(t) => t,
        };

        if let Some(l_bp) = op.postfix_binding_power() {
            if l_bp < min_bp {
                break;
            }

            match op {
                T!["["] => {
                    // Wrap the existing EXPR_SOMETHING into an EXPR for consistency.
                    ctx.start_node_at(expr_start, EXPR);
                    ctx.finish_node();

                    ctx.start_node_at(expr_start, EXPR_INDEX);
                    ctx.eat();
                    parse_expr_bp(ctx, 0);
                    match ctx.token() {
                        Some(T!("]")) => ctx.eat(),
                        Some(_) => ctx.eat_error(ParseErrorKind::ExpectedToken(T!["]"])),
                        None => ctx.add_error(ParseErrorKind::UnexpectedEof),
                    }

                    ctx.finish_node();
                }
                T!["("] => {
                    // Wrap the existing EXPR_SOMETHING into an EXPR for consistency.
                    ctx.start_node_at(expr_start, EXPR);
                    ctx.finish_node();

                    ctx.start_node_at(expr_start, EXPR_CALL);
                    parse_arg_list(ctx);
                    ctx.finish_node();
                }
                _ => unreachable!(),
            }

            continue;
        }

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
        ctx.eat();

        let expr_kind = match op {
            T!["("] => EXPR_CALL,
            _ => EXPR_BINARY,
        };

        ctx.start_node_at(expr_start, expr_kind);
        parse_expr_bp(ctx, r_bp);
        ctx.finish_node();
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_path_or_ident(ctx: &mut Context) {
    let start = ctx.checkpoint();

    expect_token!(ctx in node, T!["ident"]);

    let expr_kind = match ctx.token() {
        Some(T!["::"]) => {
            ctx.start_node_at(start, PATH);
            ctx.eat();
            loop {
                expect_token!(ctx in node, T!["ident"]);
                if let Some(T!["::"]) = ctx.token() {
                    ctx.eat();
                } else {
                    ctx.finish_node();
                    break;
                }
            }

            EXPR_PATH
        }
        Some(_) | None => EXPR_IDENT,
    };

    ctx.start_node_at(start, expr_kind);
    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_lit(ctx: &mut Context) {
    ctx.start_node(EXPR_LIT);
    parse_lit(ctx);
    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_let(ctx: &mut Context) {
    ctx.start_node(EXPR_LET);

    expect_token_eat_error!(ctx in node, T!["let"]);
    expect_token!(ctx in node, T!["ident"]);

    if !matches!(ctx.token(), Some(T!["="])) {
        ctx.finish_node();
        return;
    }

    expect_token!(ctx in node, T!["="]);

    parse_expr(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_const(ctx: &mut Context) {
    ctx.start_node(EXPR_CONST);

    expect_token_eat_error!(ctx in node, T!["const"]);
    expect_token!(ctx in node, T!["ident"]);
    expect_token!(ctx in node, T!["="]);

    parse_expr(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_block(ctx: &mut Context) {
    ctx.start_node(EXPR_BLOCK);

    expect_token_eat_error!(ctx in node, T!["{"]);

    ctx.set_statement_closed(true);
    loop {
        let token = require_token!(ctx in node);

        if token == T!["}"] {
            break;
        }

        if !ctx.statement_closed() {
            ctx.add_error(ParseErrorKind::ExpectedToken(T![";"]));
        }

        parse_stmt(ctx);
    }

    expect_token!(ctx in node, T!["}"]);

    // Blocks also don't require ";" when used as statements.
    ctx.set_statement_closed(true);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_fn(ctx: &mut Context) {
    ctx.start_node(EXPR_FN);

    expect_token_eat_error!(ctx in node, T!["fn"]);
    expect_token!(ctx in node, T!["ident"]);

    parse_param_list(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_paren(ctx: &mut Context) {
    ctx.start_node(EXPR_PAREN);
    expect_token_eat_error!(ctx in node, T!["("]);

    let token = require_token!(ctx in node);

    if matches!(token, T![")"]) {
        ctx.eat();
        ctx.finish_node();
        return;
    }

    parse_expr(ctx);

    expect_token!(ctx in node, T![")"]);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_array(ctx: &mut Context) {
    ctx.start_node(EXPR_ARRAY);

    expect_token_eat_error!(ctx in node, T!["["]);

    loop {
        let token = require_token!(ctx in node);
        if matches!(token, T!["]"]) {
            ctx.eat();
            break;
        }

        parse_expr(ctx);

        let end_token = require_token!(ctx in node);

        match end_token {
            T!["]"] => {
                ctx.eat();
                break;
            }
            T![","] => {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_closure(ctx: &mut Context) {
    ctx.start_node(EXPR_CLOSURE);

    let token = require_token!(ctx in node);

    if !matches!(token, T!["|"] | T!["||"]) {
        ctx.eat_error(ParseErrorKind::ExpectedToken(T!["|"]));
        ctx.finish_node();
        return;
    }

    parse_param_list(ctx);
    parse_expr(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_if(ctx: &mut Context) {
    ctx.start_node(EXPR_IF);

    expect_token_eat_error!(ctx in node, T!["if"]);

    parse_expr_bp(ctx, 0);
    parse_expr_block(ctx);

    if let Some(T!["else"]) = ctx.token() {
        ctx.eat();

        let token = require_token!(ctx in node);

        match token {
            T!["if"] => parse_expr_if(ctx),
            T!["{"] => parse_expr_block(ctx),
            _ => ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![T!["if"], T!["{"]])),
        }
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_loop(ctx: &mut Context) {
    ctx.start_node(EXPR_LOOP);

    expect_token_eat_error!(ctx in node, T!["loop"]);
    parse_expr_block(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_for(ctx: &mut Context) {
    ctx.start_node(EXPR_FOR);

    expect_token_eat_error!(ctx in node, T!["for"]);
    parse_pat(ctx);
    expect_token!(ctx in node, T!["in"]);
    parse_expr(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_while(ctx: &mut Context) {
    ctx.start_node(EXPR_WHILE);

    expect_token_eat_error!(ctx in node, T!["while"]);
    parse_expr(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_break(ctx: &mut Context) {
    ctx.start_node(EXPR_BREAK);

    expect_token_eat_error!(ctx in node, T!["break"]);

    if !matches!(ctx.token(), None | Some(T!["}"] | T![";"])) {
        parse_expr(ctx);
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_continue(ctx: &mut Context) {
    ctx.start_node(EXPR_CONTINUE);
    expect_token_eat_error!(ctx in node, T!["continue"]);
    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_return(ctx: &mut Context) {
    ctx.start_node(EXPR_RETURN);

    expect_token_eat_error!(ctx in node, T!["return"]);

    if !matches!(ctx.token(), None | Some(T!["}"] | T![";"])) {
        parse_expr(ctx);
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_switch(ctx: &mut Context) {
    ctx.start_node(EXPR_SWITCH);

    expect_token_eat_error!(ctx in node, T!["switch"]);
    parse_expr(ctx);
    parse_switch_arm_list(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_import(ctx: &mut Context) {
    ctx.start_node(EXPR_IMPORT);

    expect_token_eat_error!(ctx in node, T!["import"]);
    expect_token!(ctx in node, T!["lit_str"]);

    if matches!(ctx.token(), Some(T!["as"])) {
        ctx.eat();
        expect_token!(ctx in node, T!["ident"]);
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_object(ctx: &mut Context) {
    ctx.start_node(EXPR_OBJECT);

    expect_token_eat_error!(ctx in node, T!["#{"]);

    loop {
        let token = require_token!(ctx in node);
        if matches!(token, T!["}"]) {
            ctx.eat();
            break;
        }

        parse_object_field(ctx);

        let end_token = require_token!(ctx in node);

        match end_token {
            T!["}"] => {
                ctx.eat();
                break;
            }
            T![","] => {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_expr_export(ctx: &mut Context) {
    ctx.start_node(EXPR_EXPORT);

    expect_token_eat_error!(ctx in node, T!["export"]);
    parse_export_target(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_export_target(ctx: &mut Context) {
    ctx.start_node(EXPORT_TARGET);

    let token = require_token!(ctx in node);

    match token {
        T!["let"] => parse_expr_let(ctx),
        T!["const"] => parse_expr_let(ctx),
        T!["ident"] => parse_export_ident(ctx),
        _ => ctx.add_error(ParseErrorKind::UnexpectedToken),
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_export_ident(ctx: &mut Context) {
    ctx.start_node(EXPORT_IDENT);

    expect_token!(ctx in node, T!["ident"]);

    if matches!(ctx.token(), Some(T!["as"])) {
        ctx.eat();
    }

    expect_token!(ctx in node, T!["ident"]);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_pat(ctx: &mut Context) {
    ctx.start_node(PAT);

    let token = require_token!(ctx in node);

    match token {
        T!["ident"] => parse_pat_ident(ctx),
        T!["("] => parse_pat_tuple(ctx),
        _ => {
            ctx.eat_error(ParseErrorKind::UnexpectedToken);
        }
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_pat_ident(ctx: &mut Context) {
    ctx.start_node(PAT_IDENT);

    expect_token!(ctx in node, T!["ident"]);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_pat_tuple(ctx: &mut Context) {
    ctx.start_node(PAT_TUPLE);

    let start_token = require_token!(ctx in node);

    match start_token {
        T!["("] => {
            ctx.eat();
        }
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![T!["("], T!["|"]]));
            return;
        }
    }

    loop {
        let token = require_token!(ctx in node);
        if matches!(token, T![")"]) {
            ctx.eat();
            break;
        }

        parse_param(ctx);

        let end_token = require_token!(ctx in node);

        match end_token {
            T![")"] => {
                ctx.eat();
                break;
            }
            T![","] => {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_object_field(ctx: &mut Context) {
    ctx.start_node(OBJECT_FIELD);

    if !matches!(require_token!(ctx in node), T!["ident"] | T!["lit_str"]) {
        ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![
            T!["ident"],
            T!["lit_str"],
        ]));
        ctx.finish_node();
        return;
    }
    ctx.eat();

    expect_token!(ctx in node, T![":"]);

    parse_expr(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_switch_arm_list(ctx: &mut Context) {
    ctx.start_node(SWITCH_ARM_LIST);

    expect_token_eat_error!(ctx in node, T!["{"]);

    loop {
        let token = require_token!(ctx in node);
        if matches!(token, T!["}"]) {
            ctx.eat();
            break;
        }

        parse_switch_arm(ctx);

        let end_token = require_token!(ctx in node);

        match end_token {
            T!["}"] => {
                ctx.eat();
                break;
            }
            T![","] => {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_switch_arm(ctx: &mut Context) {
    ctx.start_node(SWITCH_ARM);

    let token = require_token!(ctx in node);

    if matches!(token, T!["_"]) {
        ctx.eat();
    } else {
        parse_expr(ctx);
    }

    expect_token!(ctx in node, T!["=>"]);

    parse_expr(ctx);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_param_list(ctx: &mut Context) {
    ctx.start_node(PARAM_LIST);

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

        parse_param(ctx);

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
                ctx.eat_error(ParseErrorKind::ExpectedToken(T![","]));
                break;
            }
        }
    }

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_param(ctx: &mut Context) {
    ctx.start_node(PARAM);

    expect_token!(ctx in node, T!["ident"]);

    ctx.finish_node();
}

#[instrument(level = "trace", skip(ctx))]
pub fn parse_arg_list(ctx: &mut Context) {
    ctx.start_node(ARG_LIST);

    expect_token_eat_error!(ctx in node, T!["("]);

    loop {
        let token = require_token!(ctx in node);
        if matches!(token, T![")"]) {
            ctx.eat();
            break;
        }

        parse_expr(ctx);

        let end_token = require_token!(ctx in node);

        match end_token {
            T![")"] => {
                ctx.eat();
                break;
            }
            T![","] => {
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

#[instrument(level = "trace", skip(ctx))]
pub fn parse_lit(ctx: &mut Context) {
    ctx.start_node(LIT);
    let token = require_token!(ctx in node);

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

// Binding powers based on C and python (**) operator precedence.
impl SyntaxKind {
    fn prefix_binding_power(&self) -> Option<u8> {
        let bp = match *self {
            T!["+"] | T!["-"] | T!["!"] => 22,
            _ => return None,
        };

        Some(bp)
    }

    fn infix_binding_power(&self) -> Option<(u8, u8)> {
        let bp = match *self {
            T!["+="]
            | T!["="]
            | T!["&="]
            | T!["/="]
            | T!["%="]
            | T!["*="]
            | T!["|="]
            | T!["**="]
            | T!["<<="]
            | T![">>="]
            | T!["-="]
            | T!["^="] => (1, 2),
            T!["||"] => (3, 4),
            T!["&&"] => (5, 6),
            T!["|"] => (7, 8),
            T!["^"] => (9, 10),
            T!["&"] => (10, 11),
            T!["=="] | T!["!="] => (10, 11),
            T!["<"] | T!["<="] | T![">"] | T![">="] => (12, 13),
            T!["<<"] | T![">>"] => (14, 15),
            T!["+"] | T!["-"] => (16, 17),
            T!["*"] | T!["/"] | T!["%"] => (18, 19),
            T!["**"] => (20, 21),
            T!["."] => (25, 24),
            _ => return None,
        };

        Some(bp)
    }

    fn postfix_binding_power(&self) -> Option<u8> {
        let bp = match *self {
            T!["["] | T!["("] => 23,
            _ => return None,
        };

        Some(bp)
    }
}
