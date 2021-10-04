use rhai_rowan::parser::Parser;
use test_case::test_case;

#[test_case(include_str!("../../../testdata/valid/simple.rhai"))]
#[test_case(include_str!("../../../testdata/valid/oop.rhai"))]
fn parse_valid(src: &str) {
    let parse = Parser::new(src).parse();
    assert!(parse.errors.is_empty());
    insta::assert_snapshot!(format!("{:#?}", parse.into_syntax()));
}
