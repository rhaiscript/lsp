use logos::Logos;
use rhai::Token;
use rhai_rowan::ast::{AstNode, Expr, Rhai};
use rhai_rowan::Parser;
use rhai_rowan::syntax::SyntaxKind;

#[test]
fn switch_multiple_values () {
    let script = "switch x { 1 | 2 | 3 => 1, _ => 3 }";
    let parse = Parser::new(script).parse_script();
    assert!(parse.errors.is_empty(), "{:#?}", parse);
    let rhai = Rhai::cast(parse.into_syntax()).unwrap();
    let stmt = rhai.statements().next().expect("one statement");
    let expr = stmt.item().and_then(|item| item.expr()).expect("an expression");

    // switch x { 1 | 2 | 3 => 1, _ => 3 }
    let Expr::Switch(switch) = expr else {
        panic!("expected a switch expression, got {expr:#?}");
    };

    // 1 | 2 | 3 => 1
    let arm = switch
        .switch_arm_list()
        .and_then(|list| list.arms().next())
        .expect("first switch arm");
    // 1 | 2 | 3
    let pattern = arm.syntax().first_child().expect("a pattern");

    // Should be Syntax::SWITCH_ARM_PATTERNS or something similar for 1 | 2 | 3 not regular expr
    assert!(!matches!(pattern.kind(), SyntaxKind::EXPR), "pattern was {pattern:#?}");
}

#[test]
fn reserved_left_pipe() {
    let mut lexer = SyntaxKind::lexer("<|");
    let first = lexer.next();
    let second = lexer.next();
    assert!(first.is_some());
    assert!(first.unwrap().is_reserved_keyword(), "first token is not reserved: {first:#?}");
    assert!(second.is_none(), "second token was {second:#?}");
}

#[test]
fn reserved_right_pipe() {
    let mut lexer = SyntaxKind::lexer("|>");
    let first = lexer.next();
    let second = lexer.next();
    assert!(first.is_some());
    assert!(first.unwrap().is_reserved_keyword(), "first token is not reserved: {first:#?}");
    assert!(second.is_none(), "second token was {second:#?}");
}

#[test]
fn module_doc() {
    let parse = Parser::new("//! Module docs.\n").parse_script();
    assert!(parse.errors.is_empty());
    let rhai = Rhai::cast(parse.into_syntax()).unwrap();
    assert_eq!(rhai.script_docs(), "Module docs.\n");

    let mut lexer = SyntaxKind::lexer("//!");
    let first = lexer.next();
    assert!(first.is_some());
    let first = first.unwrap();
    assert_ne!(first, SyntaxKind::COMMENT_LINE, "//! should be module documentation: {first:#?}");
}