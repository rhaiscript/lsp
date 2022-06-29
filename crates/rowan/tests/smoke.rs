use rhai_rowan::parser::Parser;
use test_case::test_case;

#[test_case("simple", include_str!("../../../testdata/valid/simple.rhai"))]
#[test_case("oop", include_str!("../../../testdata/valid/oop.rhai"))]
fn parse_valid(name: &str, src: &str) {
    let parse = Parser::new(src).parse_script();
    assert!(parse.errors.is_empty());

    insta::with_settings!(
        { snapshot_suffix => name },
        {
            insta::assert_snapshot!(format!("{:#?}", parse.into_syntax()));
        }
    );
}
