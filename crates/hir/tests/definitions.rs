use rhai_hir::Hir;
use rhai_rowan::parser::Parser;

#[test]
fn test_global_definition() {
    let root_src = r#"
global::print();
"#;

    let global_src = r#"
module global;

fn print();
"#;

    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///global.d.rhai".parse().unwrap(),
        &Parser::new(global_src).parse_def().into_syntax(),
    );

    hir.resolve_references();

    assert!(hir.errors().is_empty());
}

#[test]
fn test_static_module() {
    let root_src = r#"
print("hello");
"#;

    let global_src = r#"
module static;

fn print();
"#;

    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///static.d.rhai".parse().unwrap(),
        &Parser::new(global_src).parse_def().into_syntax(),
    );

    hir.resolve_references();

    assert!(hir.errors().is_empty());
}

#[test]
fn test_define_file() {
    let root_src = r#"
print("hello");
"#;

    let global_src = r#"
module;

fn print();
"#;

    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///root.d.rhai".parse().unwrap(),
        &Parser::new(global_src).parse_def().into_syntax(),
    );

    hir.resolve_references();

    assert!(hir.errors().is_empty());
}

#[test]
fn test_define_file_explicitly() {
    let root_src = r#"
print("hello");
"#;

    let global_src = r#"
module "./root.rhai";

fn print();
"#;

    let mut hir = Hir::new();

    hir.add_source(
        &"test:///root.rhai".parse().unwrap(),
        &Parser::new(root_src).parse_script().into_syntax(),
    );
    hir.add_source(
        &"test:///root.d.rhai".parse().unwrap(),
        &Parser::new(global_src).parse_def().into_syntax(),
    );

    hir.resolve_references();

    assert!(hir.errors().is_empty());
}

