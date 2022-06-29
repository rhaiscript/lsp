use rhai_rowan::parser::Parser;
use test_case::test_case;

#[test_case("simple", include_str!("../../../testdata/valid/simple.rhai"))]
#[test_case("array", include_str!("../../../testdata/valid/array.rhai"))]
#[test_case("assignment", include_str!("../../../testdata/valid/assignment.rhai"))]
#[test_case("comments", include_str!("../../../testdata/valid/comments.rhai"))]
#[test_case("fibonacci", include_str!("../../../testdata/valid/fibonacci.rhai"))]
#[test_case("for1", include_str!("../../../testdata/valid/for1.rhai"))]
#[test_case("for2", include_str!("../../../testdata/valid/for2.rhai"))]
#[test_case("function_decl1", include_str!("../../../testdata/valid/function_decl1.rhai"))]
#[test_case("function_decl2", include_str!("../../../testdata/valid/function_decl2.rhai"))]
#[test_case("function_decl3", include_str!("../../../testdata/valid/function_decl3.rhai"))]
#[test_case("function_decl4", include_str!("../../../testdata/valid/function_decl4.rhai"))]
#[test_case("if1", include_str!("../../../testdata/valid/if1.rhai"))]
#[test_case("if2", include_str!("../../../testdata/valid/if2.rhai"))]
#[test_case("loop", include_str!("../../../testdata/valid/loop.rhai"))]
#[test_case("mat_mul", include_str!("../../../testdata/valid/mat_mul.rhai"))]
#[test_case("module", include_str!("../../../testdata/valid/module.rhai"))]
#[test_case("oop", include_str!("../../../testdata/valid/oop.rhai"))]
#[test_case("op1", include_str!("../../../testdata/valid/op1.rhai"))]
#[test_case("op2", include_str!("../../../testdata/valid/op2.rhai"))]
#[test_case("op3", include_str!("../../../testdata/valid/op3.rhai"))]
#[test_case("primes", include_str!("../../../testdata/valid/primes.rhai"))]
#[test_case("speed_test", include_str!("../../../testdata/valid/speed_test.rhai"))]
#[test_case("string", include_str!("../../../testdata/valid/string.rhai"))]
#[test_case("strings_map", include_str!("../../../testdata/valid/strings_map.rhai"))]
#[test_case("switch", include_str!("../../../testdata/valid/switch.rhai"))]
#[test_case("while", include_str!("../../../testdata/valid/while.rhai"))]
#[test_case("char", include_str!("../../../testdata/valid/char.rhai"))]
fn parse_valid(name: &str, src: &str) {
    let parse = Parser::new(src).parse_script();
    assert!(parse.errors.is_empty(), "{:#?}", parse.errors);

    insta::with_settings!(
        { snapshot_suffix => name },
        {
            insta::assert_snapshot!(format!("{:#?}", parse.into_syntax()));
        }
    );

    rhai::Engine::new()
        .set_max_expr_depths(0, 0)
        .compile(src)
        .unwrap();
}
