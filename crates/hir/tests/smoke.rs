use rhai_hir::Hir;
use rhai_rowan::parser::Parser;

#[test]
fn references() {
    let src = r#"
/// Test doc
let aa = #{
};

aa;    
"#;
    let parse = Parser::new(src).parse();
    assert!(parse.errors.is_empty());

    let mut hir = Hir::new();

    hir.add_module_from_syntax("references", &parse.clone_syntax());
    hir.resolve_references();

    let m = hir.get_module("references").unwrap();

    let (aa_sym, aa_data) = m.symbols().skip(2).next().unwrap();

    let aa_decl = m.symbols_by_name("aa").next().unwrap();
    let aa_decl_data = &m[aa_decl];

    assert_eq!(
        aa_data
            .kind
            .as_reference()
            .as_ref()
            .unwrap()
            .target
            .as_ref()
            .unwrap()
            .as_symbol()
            .unwrap(),
        &aa_decl
    );
    assert!(aa_decl_data
        .kind
        .as_decl()
        .unwrap()
        .references
        .contains(&aa_sym));
}

#[test]
fn nested_expr() {
    let src = r#"
let a = 1;
a + a + a * a
"#;

    let parse = Parser::new(src).parse();
    assert!(parse.errors.is_empty());

    let mut hir = Hir::new();

    hir.add_module_from_syntax("nested_expr", &parse.clone_syntax());
    hir.resolve_references();

    let m = hir.get_module("nested_expr").unwrap();

    let aa_decl = m.symbols_by_name("a").next().unwrap();
    let aa_data = &m[aa_decl].kind.as_decl().unwrap();

    assert_eq!(aa_data.references.len(), 4);
}
