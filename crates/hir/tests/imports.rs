use rhai_hir::Hir;
use rhai_rowan::parser::Parser;

#[test]
fn test_relative_import() {
    let root_src = r#"
import "./module.rhai" as m;

m::x;
"#;

    let module_src = r#"
export const x = 1;
"#;

    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///module.rhai".parse().unwrap(),
        &Parser::new(module_src).parse_script().into_syntax(),
    );

    hir.resolve_references();

    assert!(hir.errors().is_empty());
}

#[test]
fn test_import_sub_modules() {
    let root_src = r#"
import "./foo.rhai" as foo;

foo::bar::baz;
"#;

    let foo_src = r#"
import "./bar.rhai" as bar;
"#;

    let bar_src = r#"
export const baz = 1;
"#;

    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///foo.rhai".parse().unwrap(),
        &Parser::new(foo_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///bar.rhai".parse().unwrap(),
        &Parser::new(bar_src).parse_script().into_syntax(),
    );

    hir.resolve_references();

    assert!(hir.errors().is_empty());
}

#[test]
fn test_missing_modules() {
    let root_src = r#"
import "./foo.rhai" as foo;

"#;


    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );

    hir.resolve_references();

    assert_eq!(hir.missing_modules().len(), 1);
}

