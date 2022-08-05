//! A collection modular parser functions for Rhai.

#![deny(unreachable_patterns)]

use logos::Logos;

use super::context::Context;
use crate::parser::ParseErrorKind;
use crate::syntax::{SyntaxKind, SyntaxKind::*};
use crate::T;

/// Require a token or else add an unexpected EOF error and return.
///
/// # Usage Example
///
/// ```ignore
/// let token = require_token!(ctx);
///
/// // Or also call [`Context::finish_node`] on error before returning.
/// let token = require_token(ctx in node);
/// ```
macro_rules! require_token {
    ($ctx:ident) => {
        match $ctx.token() {
            Some(t) => t,
            None => return $ctx.eat_error(ParseErrorKind::UnexpectedEof),
        }
    };
    ($ctx:ident in node) => {
        require_token!($ctx in nodes(1))
    };
    ($ctx:ident in nodes($count:literal)) => {
        match $ctx.token() {
            Some(t) => t,
            None => {
                for _ in 0..$count {
                    $ctx.finish_node();
                }
                $ctx.eat_error(ParseErrorKind::UnexpectedEof);
                return;
            }
        }
    };
}

/// Expect a given a token or else add an error describing the expected token and return.
///
/// # Usage Example
///
/// ```ignore
/// expect_token!(ctx, T!["="]);
///
/// // Or also call [`Context::finish_node`] on error before returning.
/// expect_token!(ctx in node, T!["="]);
/// ```
///
/// It will not cause the current token to be eaten on error.
macro_rules! expect_token {
    ($ctx:ident in node, $($token:tt)*) => {
        match $ctx.token() {
            Some($($token)*) => {
                $ctx.eat();
            }
            _ => {
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

pub mod def;
pub mod ty;

impl<'src> super::Parser<'src> {
    /// Parse Rhai code with [`parse_rhai`], and finish the parser.
    pub fn parse_script(mut self) -> super::Parse {
        self.execute(parse_rhai);
        self.finish()
    }
}

/// Parse a Rhai file.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_rhai(ctx: &mut Context) {
    ctx.start_node(RHAI);
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

    ctx.finish_node();
}

/// Parse a shebang like `#!something`, typically at the start of files.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_shebang(ctx: &mut Context) {
    let token = require_token!(ctx);

    if !matches!(token, SHEBANG) {
        return ctx.eat_error(ParseErrorKind::ExpectedToken(SHEBANG));
    }

    ctx.eat();
}

/// Parse a statement.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_stmt(ctx: &mut Context) {
    ctx.start_node(STMT);
    let token = require_token!(ctx in node);
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

/// Parse an item.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_item(ctx: &mut Context) {
    ctx.start_node(ITEM);

    // Parse doc comments if any.
    while matches!(
        require_token!(ctx in node),
        COMMENT_BLOCK_DOC | COMMENT_LINE_DOC
    ) {
        ctx.start_node(DOC);
        ctx.eat();
        ctx.finish_node();
    }

    parse_expr(ctx);

    ctx.finish_node();
}

/// Parse an expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr(ctx: &mut Context) {
    parse_expr_bp(ctx, 0);
}

/// Pratt-based expression parsing.
///
/// `min_bp` is the current minimum binding power
/// in the expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_expr_bp(ctx: &mut Context, min_bp: u8) {
    ctx.start_node(EXPR);

    let expr_start = ctx.checkpoint();

    let token = require_token!(ctx in node);

    if let Some(T!["."]) = ctx.previous_token() {
        if token != IDENT {
            ctx.add_error(ParseErrorKind::ExpectedToken(IDENT));
        }
    }

    // Handle "standalone" expressions, and
    // unary operators.
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
        T!["throw"] => {
            parse_expr_throw(ctx);
            ctx.finish_node();
            return;
        }
        T!["#{"] => {
            parse_expr_object(ctx);
            if ctx.token().is_some() {
                // This can be part of a binary expression,
                // but it's also block-like.
                //
                // To disambiguate, we check whether the next token
                // is a binary operator or not.
                if ctx.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["fn"] | T!["private"] => {
            parse_expr_fn(ctx);
            ctx.finish_node();
            return;
        }
        T!["|"] | T!["||"] => {
            // boolean "or" is a special case that has to be handled here.
            parse_expr_closure(ctx);
            ctx.finish_node();
            return;
        }
        T!["if"] => {
            parse_expr_if(ctx);

            if ctx.token().is_some() {
                // This can be part of a binary expression,
                // but it's also block-like.
                //
                // To disambiguate, we check whether the next token
                // is a binary operator or not.
                if ctx.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["loop"] => {
            parse_expr_loop(ctx);
            if ctx.token().is_some() && ctx.infix_binding_power().is_none() {
                ctx.finish_node();
                return;
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
            if ctx.token().is_some() {
                // This can be part of a binary expression,
                // but it's also block-like.
                //
                // To disambiguate, we check whether the next token
                // is a binary operator or not.
                if ctx.infix_binding_power().is_none() {
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
        T!["try"] => {
            parse_expr_try(ctx);
            ctx.finish_node();
            return;
        }
        T!["{"] => {
            parse_expr_block(ctx);
            if ctx.token().is_some() {
                // This can be part of a binary expression,
                // but it's also block-like.
                //
                // To disambiguate, we check whether the next token
                // is a binary operator or not.
                if ctx.infix_binding_power().is_none() {
                    ctx.finish_node();
                    return;
                }
            }
        }
        T!["("] => parse_expr_paren(ctx),
        T!["["] => parse_expr_array(ctx),
        LIT_INT | LIT_FLOAT | LIT_BOOL | LIT_STR | LIT_CHAR | __TEMP_STR_TEMPLATE_START => {
            parse_expr_lit(ctx);
        }
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
        }
    }

    loop {
        // We treat everything as expressions, statements are simply expressions
        // delimited by `;`.
        //
        // Here we list all the cases when expressions have to end no matter what.
        let op_token = match ctx.token() {
            Some(
                T![";"] | T![","] | T!["{"] | T!["}"] | T![")"] | T!["]"] | T!["=>"] | T!["as"],
            )
            | None => break,
            Some(T!["if"]) if ctx.switch_pat_expr() => break,
            Some(t) => t,
        };

        if let Some(l_bp) = op_token.postfix_binding_power() {
            if l_bp < min_bp {
                break;
            }

            // Wrap the existing EXPR_SOMETHING into an EXPR for consistency.
            ctx.start_node_at(expr_start, EXPR);
            ctx.finish_node();

            match op_token {
                T!["["] | T!["?["] => {
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
                    ctx.start_node_at(expr_start, EXPR_CALL);
                    parse_arg_list(ctx);
                    ctx.finish_node();
                }
                _ => unreachable!(),
            }

            continue;
        }

        let (l_bp, r_bp) = match ctx.infix_binding_power() {
            Some(bp) => bp,
            None => {
                ctx.add_error(ParseErrorKind::UnexpectedToken);

                if op_token == T!["ident"] || op_token.is_reserved_keyword() {
                    ctx.start_node_at(expr_start, EXPR);
                    ctx.finish_node();

                    ctx.start_node_at(expr_start, EXPR_BINARY);
                    ctx.eat();
                    ctx.finish_node();
                }
                break;
            }
        };
        if l_bp < min_bp {
            break;
        }
        // Wrap the existing EXPR_SOMETHING into an EXPR for consistency.
        ctx.start_node_at(expr_start, EXPR);
        ctx.finish_node();

        if ctx.token().map_or(false, |t| t.is_reserved_keyword()) {
            ctx.eat_as(T!["ident"]);
        } else {
            ctx.eat();
        }

        ctx.start_node_at(expr_start, EXPR_BINARY);
        parse_expr_bp(ctx, r_bp);
        ctx.finish_node();
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_throw(ctx: &mut Context) {
    ctx.start_node(EXPR_THROW);

    expect_token!(ctx in node, T!["throw"]);
    parse_expr(ctx);

    ctx.finish_node();
}

/// Parse a path such as `a::b` or an identifier.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
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

/// Parse an ident expression ([`ExprIdent`](`crate::ast::generated::ExprIdent`)).
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_ident(ctx: &mut Context) {
    ctx.start_node(EXPR_IDENT);
    expect_token!(ctx in node, T!["ident"]);
    ctx.finish_node();
}

/// Parse a literal expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_lit(ctx: &mut Context) {
    ctx.start_node(EXPR_LIT);
    parse_lit(ctx);
    ctx.finish_node();
}

/// Parse a `let` expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_let(ctx: &mut Context) {
    ctx.start_node(EXPR_LET);

    expect_token!(ctx in node, T!["let"]);
    expect_token!(ctx in node, T!["ident"]);

    if !matches!(ctx.token(), Some(T!["="])) {
        ctx.finish_node();
        return;
    }

    expect_token!(ctx in node, T!["="]);

    parse_expr(ctx);

    ctx.finish_node();
}

/// Parse a `const` expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_const(ctx: &mut Context) {
    ctx.start_node(EXPR_CONST);

    expect_token!(ctx in node, T!["const"]);
    expect_token!(ctx in node, T!["ident"]);
    expect_token!(ctx in node, T!["="]);

    parse_expr(ctx);

    ctx.finish_node();
}

/// Parse a block expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_block(ctx: &mut Context) {
    ctx.start_node(EXPR_BLOCK);

    expect_token!(ctx in node, T!["{"]);

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

/// Parse a `fn` expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_fn(ctx: &mut Context) {
    ctx.start_node(EXPR_FN);

    if let Some(T!["private"]) = ctx.token() {
        ctx.eat();
    }

    expect_token!(ctx in node, T!["fn"]);
    expect_token!(ctx in node, T!["ident"]);

    parse_param_list(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

/// Parse a parenthesized expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_paren(ctx: &mut Context) {
    ctx.start_node(EXPR_PAREN);
    expect_token!(ctx in node, T!["("]);

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

/// Parse an array expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_array(ctx: &mut Context) {
    ctx.start_node(EXPR_ARRAY);

    expect_token!(ctx in node, T!["["]);

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

/// Parse a closure expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
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

/// Parse an "if" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_if(ctx: &mut Context) {
    ctx.start_node(EXPR_IF);

    expect_token!(ctx in node, T!["if"]);

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

/// Parse a "loop" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_loop(ctx: &mut Context) {
    ctx.start_node(EXPR_LOOP);

    expect_token!(ctx in node, T!["loop"]);
    parse_expr_block(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_for(ctx: &mut Context) {
    ctx.start_node(EXPR_FOR);

    expect_token!(ctx in node, T!["for"]);
    parse_pat(ctx);
    expect_token!(ctx in node, T!["in"]);
    parse_expr(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

/// Parse a "while" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_while(ctx: &mut Context) {
    ctx.start_node(EXPR_WHILE);

    expect_token!(ctx in node, T!["while"]);
    parse_expr(ctx);
    parse_expr_block(ctx);

    ctx.finish_node();
}

/// Parse a "break" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_break(ctx: &mut Context) {
    ctx.start_node(EXPR_BREAK);

    expect_token!(ctx in node, T!["break"]);

    if !matches!(ctx.token(), None | Some(T!["}"] | T![";"])) {
        parse_expr(ctx);
    }

    ctx.finish_node();
}

/// Parse a "continue" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_continue(ctx: &mut Context) {
    ctx.start_node(EXPR_CONTINUE);
    expect_token!(ctx in node, T!["continue"]);
    ctx.finish_node();
}

/// Parse a "return" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_return(ctx: &mut Context) {
    ctx.start_node(EXPR_RETURN);

    expect_token!(ctx in node, T!["return"]);

    if !matches!(ctx.token(), None | Some(T!["}"] | T![";"])) {
        parse_expr(ctx);
    }

    ctx.finish_node();
}

/// Parse a "switch" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_switch(ctx: &mut Context) {
    ctx.start_node(EXPR_SWITCH);

    expect_token!(ctx in node, T!["switch"]);
    parse_expr(ctx);
    parse_switch_arm_list(ctx);

    ctx.finish_node();
}

/// Parse an "import" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_import(ctx: &mut Context) {
    ctx.start_node(EXPR_IMPORT);

    expect_token!(ctx in node, T!["import"]);
    parse_expr(ctx);

    if matches!(ctx.token(), Some(T!["as"])) {
        ctx.eat();
        expect_token!(ctx in node, T!["ident"]);
    }

    ctx.finish_node();
}

/// Parse an object literal expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_object(ctx: &mut Context) {
    ctx.start_node(EXPR_OBJECT);

    expect_token!(ctx in node, T!["#{"]);

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

/// Parse an "export" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_export(ctx: &mut Context) {
    ctx.start_node(EXPR_EXPORT);

    expect_token!(ctx in node, T!["export"]);
    parse_export_target(ctx);

    ctx.finish_node();
}

/// Parse a "try" expression.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
pub fn parse_expr_try(ctx: &mut Context) {
    ctx.start_node(EXPR_TRY);

    expect_token!(ctx in node, T!["try"]);
    parse_expr_block(ctx);
    expect_token!(ctx in node, T!["catch"]);
    let token = require_token!(ctx in node);

    if token == T!["("] {
        parse_param_list(ctx);
    }

    parse_expr_block(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_export_target(ctx: &mut Context) {
    ctx.start_node(EXPORT_TARGET);

    let token = require_token!(ctx in node);

    match token {
        T!["let"] => parse_expr_let(ctx),
        T!["const"] => parse_expr_const(ctx),
        T!["ident"] => parse_export_ident(ctx),
        _ => ctx.add_error(ParseErrorKind::UnexpectedToken),
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_export_ident(ctx: &mut Context) {
    ctx.start_node(EXPORT_IDENT);

    expect_token!(ctx in node, T!["ident"]);

    if matches!(ctx.token(), Some(T!["as"])) {
        ctx.eat();
    }

    expect_token!(ctx in node, T!["ident"]);

    ctx.finish_node();
}

/// Parse a pattern.
#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
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

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_pat_ident(ctx: &mut Context) {
    ctx.start_node(PAT_IDENT);

    expect_token!(ctx in node, T!["ident"]);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_pat_tuple(ctx: &mut Context) {
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

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_object_field(ctx: &mut Context) {
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

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_switch_arm_list(ctx: &mut Context) {
    ctx.start_node(SWITCH_ARM_LIST);

    expect_token!(ctx in node, T!["{"]);

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

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_switch_arm(ctx: &mut Context) {
    ctx.start_node(SWITCH_ARM);

    let token = require_token!(ctx in node);

    if matches!(token, T!["_"]) {
        ctx.eat();
    } else {
        ctx.set_switch_pat_expr(true);
        parse_expr(ctx);
        ctx.set_switch_pat_expr(false);
    }

    let token = require_token!(ctx in node);

    if matches!(token, T!["if"]) {
        parse_switch_arm_condition(ctx);
    }

    expect_token!(ctx in node, T!["=>"]);

    parse_expr(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_switch_arm_condition(ctx: &mut Context) {
    ctx.start_node(SWITCH_ARM_CONDITION);

    expect_token!(ctx in node, T!["if"]);
    parse_expr(ctx);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_param_list(ctx: &mut Context) {
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
                ctx.add_error(ParseErrorKind::ExpectedToken(T![","]));
            }
        }
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_param(ctx: &mut Context) {
    ctx.start_node(PARAM);

    expect_token!(ctx in node, T!["ident"]);

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_arg_list(ctx: &mut Context) {
    ctx.start_node(ARG_LIST);

    expect_token!(ctx in node, T!["("]);

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

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_lit(ctx: &mut Context) {
    ctx.start_node(LIT);
    let token = require_token!(ctx in node);

    match token {
        LIT_INT | LIT_FLOAT | LIT_BOOL | LIT_STR | LIT_CHAR => {
            ctx.eat();
        }
        __TEMP_STR_TEMPLATE_START => parse_lit_str_template(ctx),
        _ => {
            ctx.eat_error(ParseErrorKind::ExpectedOneOfTokens(vec![
                LIT_INT, LIT_FLOAT, LIT_BOOL, LIT_STR, LIT_CHAR,
            ]));
        }
    }

    ctx.finish_node();
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(ctx))]
fn parse_lit_str_template(ctx: &mut Context) {
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Logos)]
    #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
    pub enum LitStrTemplateToken {
        #[token("${")]
        INTERPOLATION_START,

        #[token(r#"``"#)]
        ESCAPED_BACKTICK,

        #[token("`")]
        END_BACKTICK,

        #[error]
        OTHER,
    }
    use LitStrTemplateToken::*;

    ctx.start_node(LIT_STR_TEMPLATE);

    let token = require_token!(ctx in node);

    if token != __TEMP_STR_TEMPLATE_START {
        ctx.add_error(ParseErrorKind::UnexpectedToken);
        ctx.finish_node();
        return;
    }

    let mut str_lex = LitStrTemplateToken::lexer(ctx.remainder());
    let mut len = 0;
    let mut had_interpolated = false;

    while let Some(token) = str_lex.next() {
        match token {
            INTERPOLATION_START => {
                if len > 0 {
                    if had_interpolated {
                        ctx.token_raw().unwrap();
                        ctx.bump(len - ctx.slice().len());
                    } else {
                        ctx.bump(len);
                    }
                }

                ctx.eat_as(LIT_STR);
                expect_token!(ctx in node, T!["${"]);
                ctx.set_statement_closed(true);
                ctx.start_node(LIT_STR_TEMPLATE_INTERPOLATION);
                loop {
                    let token = require_token!(ctx in nodes(2));

                    if token == T!["}"] {
                        break;
                    }

                    if !ctx.statement_closed() {
                        ctx.add_error(ParseErrorKind::ExpectedToken(T![";"]));
                    }

                    parse_stmt(ctx);
                }
                ctx.finish_node();
                expect_token!(ctx in node, T!["}"]);
                str_lex = LitStrTemplateToken::lexer(ctx.remainder());
                len = 0;
                had_interpolated = true;
            }
            END_BACKTICK => {
                len += str_lex.slice().len();
                // If there was an interpolated part
                // the current context might not yet
                // have a token to bump.
                if had_interpolated {
                    ctx.token_raw().unwrap();
                    ctx.bump(len - ctx.slice().len());
                } else {
                    ctx.bump(len);
                }

                ctx.eat_as(LIT_STR);
                ctx.finish_node();
                return;
            }
            OTHER | ESCAPED_BACKTICK => {
                len += str_lex.slice().len();
            }
        }
    }
    ctx.add_error(ParseErrorKind::InvalidOrUnclosedString);

    ctx.finish_node();
}

impl SyntaxKind {
    #[must_use]
    pub fn infix_binding_power(self) -> Option<(u8, u8)> {
        let bp = match self {
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
            T!["||"] | T!["|"] | T!["^"] => (30, 31),
            T!["&&"] | T!["&"] => (60, 61),
            T!["=="] | T!["!="] => (90, 91),
            T!["in"] => (110, 111),
            T!["<"] | T!["<="] | T![">"] | T![">="] => (130, 131),
            T!["??"] => (135, 136),
            T![".."] | T!["..="] => (140, 141),
            T!["+"] | T!["-"] => (150, 151),
            T!["*"] | T!["/"] | T!["%"] => (180, 181),
            T!["**"] => (191, 190),
            T!["<<"] | T![">>"] => (210, 211),
            T!["."] | T!["?."] => (254, 255),
            _ => return None,
        };

        Some(bp)
    }

    #[must_use]
    pub fn prefix_binding_power(self) -> Option<u8> {
        let bp = match self {
            T!["+"] | T!["-"] | T!["!"] => 252,
            _ => return None,
        };

        Some(bp)
    }

    #[must_use]
    pub fn postfix_binding_power(self) -> Option<u8> {
        let bp = match self {
            T!["?["] | T!["["] | T!["("] => 253,
            _ => return None,
        };

        Some(bp)
    }
}
