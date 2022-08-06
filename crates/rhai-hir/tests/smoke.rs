use rhai_hir::Hir;
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
#[test_case("throw_try_catch", include_str!("../../../testdata/valid/throw_try_catch.rhai"))]
fn construct_hir(_name: &str, src: &str) {
    let parse = Parser::new(src).parse_script();
    assert!(parse.errors.is_empty(), "{:#?}", parse.errors);

    let mut hir = Hir::new();
    hir.add_source(
        &"test:///example.rhai".parse().unwrap(),
        &parse.into_syntax(),
    );
}

#[test]
fn add_and_remove_sources() {
    fn add(hir: &mut Hir, name: &str, src: &str) {
        let parse = Parser::new(src).parse_script();
        assert!(parse.errors.is_empty(), "{:#?}", parse.errors);
        hir.add_source(
            &format!("test:///{name}.rhai").parse().unwrap(),
            &parse.into_syntax(),
        );
    }

    fn remove(hir: &mut Hir, name: &str) {
        hir.remove_source(
            hir.source_by_url(&format!("test:///{name}.rhai").parse().unwrap())
                .unwrap(),
        )
    }

    let sources = [
        (
            "simple",
            include_str!("../../../testdata/valid/simple.rhai"),
        ),
        ("array", include_str!("../../../testdata/valid/array.rhai")),
        (
            "assignment",
            include_str!("../../../testdata/valid/assignment.rhai"),
        ),
        (
            "comments",
            include_str!("../../../testdata/valid/comments.rhai"),
        ),
        (
            "fibonacci",
            include_str!("../../../testdata/valid/fibonacci.rhai"),
        ),
        ("for1", include_str!("../../../testdata/valid/for1.rhai")),
        ("for2", include_str!("../../../testdata/valid/for2.rhai")),
        (
            "function_decl1",
            include_str!("../../../testdata/valid/function_decl1.rhai"),
        ),
        (
            "function_decl2",
            include_str!("../../../testdata/valid/function_decl2.rhai"),
        ),
        (
            "function_decl3",
            include_str!("../../../testdata/valid/function_decl3.rhai"),
        ),
        (
            "function_decl4",
            include_str!("../../../testdata/valid/function_decl4.rhai"),
        ),
        ("if1", include_str!("../../../testdata/valid/if1.rhai")),
        ("if2", include_str!("../../../testdata/valid/if2.rhai")),
        ("loop", include_str!("../../../testdata/valid/loop.rhai")),
        (
            "mat_mul",
            include_str!("../../../testdata/valid/mat_mul.rhai"),
        ),
        (
            "module",
            include_str!("../../../testdata/valid/module.rhai"),
        ),
        ("oop", include_str!("../../../testdata/valid/oop.rhai")),
        ("op1", include_str!("../../../testdata/valid/op1.rhai")),
        ("op2", include_str!("../../../testdata/valid/op2.rhai")),
        ("op3", include_str!("../../../testdata/valid/op3.rhai")),
        (
            "primes",
            include_str!("../../../testdata/valid/primes.rhai"),
        ),
        (
            "speed_test",
            include_str!("../../../testdata/valid/speed_test.rhai"),
        ),
        (
            "string",
            include_str!("../../../testdata/valid/string.rhai"),
        ),
        (
            "strings_map",
            include_str!("../../../testdata/valid/strings_map.rhai"),
        ),
        (
            "switch",
            include_str!("../../../testdata/valid/switch.rhai"),
        ),
        ("while", include_str!("../../../testdata/valid/while.rhai")),
        ("char", include_str!("../../../testdata/valid/char.rhai")),
        (
            "throw_try_catch",
            include_str!("../../../testdata/valid/throw_try_catch.rhai"),
        ),
        (
            "for2_2",
            include_str!("../../../testdata/benchmarks/for2.rhai"),
        ),
    ];

    let mut hir = Hir::new();

    // In some cases reference issues appeared only
    // after the second time a symbol was attempted
    // to be resolved.
    //
    // So we repeat some ops a few times to
    // potentially catch more errors.
    for _ in 0..5 {
        for (name, source) in sources {
            add(&mut hir, name, source);
        }

        hir.resolve_references();

        for (name, _) in sources {
            remove(&mut hir, name);
        }

        hir.resolve_references();
    }
}
