use rhai_rowan::parser::Parser;
use test_case::test_case;

#[test_case(include_str!("../../../testdata/valid/simple.rhai"))]
#[test_case(include_str!("../../../testdata/valid/oop.rhai"))]
#[test_case(include_str!("../../../testdata/valid/module-example.rhai"))]
fn parse_valid(src: &str) {
    let parse = Parser::new(src).parse();
    assert!(parse.errors.is_empty());
    insta::assert_snapshot!(format!("{:#?}", parse.into_syntax()));
}

// #[test_case(include_str!("../../../testdata/valid/module-example.d.rhai"))]
// #[test_case(include_str!("../../../testdata/valid/module-global.d.rhai"))]
// #[test_case(include_str!("../../../testdata/valid/module-named.d.rhai"))]
// #[test_case(include_str!("../../../testdata/valid/module-static.d.rhai"))]
#[test]
fn parse_valid_definitions() {
    let parse = Parser::new(include_str!("../../../testdata/valid/module-static.d.rhai")).parse_def();
    assert!(parse.errors.is_empty());
    insta::assert_snapshot!(format!("{:#?}", parse.into_syntax()));
}
