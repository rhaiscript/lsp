use rhai_hir::Hir;
use rhai_rowan::{parser::Parser, util::src_cursor_offset};

#[test]
fn test_visible_symbols_from_offset() {
    let (offset, src) = src_cursor_offset(
        r#"fn foo() {}
$$
let bar = 3;


"#,
    );

    let mut hir = Hir::new();

    let url = "test:///global.rhai".parse().unwrap();

    hir.add_source(&url, &Parser::new(&src).parse_def().into_syntax());

    hir.resolve_all();

    assert!(hir
        .visible_symbols_from_offset(hir.source_by_url(&url).unwrap(), offset, false)
        .next()
        .is_some())
}

#[test]
fn test_visible_import() {
    let (offset, src) = src_cursor_offset(
        r#"import "module" as m;

    $$
"#,
    );

    let mut hir = Hir::new();

    let url = "test:///global.rhai".parse().unwrap();

    hir.add_source(&url, &Parser::new(&src).parse_def().into_syntax());

    hir.resolve_all();

    assert!(hir
        .visible_symbols_from_offset(hir.source_by_url(&url).unwrap(), offset, false)
        .find_map(|symbol| hir[symbol].kind.as_import().and_then(|d| d.alias))
        .is_some())
}
