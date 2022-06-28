use rhai_rowan::parser::Parser;
use test_case::test_case;

#[test_case(include_str!("../../../rhai/scripts/array.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/assignment.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/comments.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/fibonacci.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/for1.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/for2.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/function_decl1.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/function_decl2.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/function_decl3.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/function_decl4.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/if1.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/if2.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/loop.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/mat_mul.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/module.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/oop.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/op1.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/op2.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/op3.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/primes.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/speed_test.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/string.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/strings_map.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/switch.rhai"))]
#[test_case(include_str!("../../../rhai/scripts/while.rhai"))]
fn parse_rhai_upstream(src: &str) {
    let parse = Parser::new(src).parse_script();
    assert!(parse.errors.is_empty());
    // TODO(tests)
    // insta::assert_snapshot!(format!("{:#?}", parse.into_syntax()));
}
