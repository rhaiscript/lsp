use crate::{
    parser::{Operator, Parser},
    util::{src_cursor_offset, src_cursor_offsets},
};

use super::*;

#[test]
fn test_query_field_access_empty() {
    let (offset, src) = src_cursor_offset(
        r#"
            foo.   $$
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(q.is_field_access());
}

#[test]
fn test_query_field_access_ident() {
    let (offset, src) = src_cursor_offset(
        r#"
            foo.   id$$ent
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(q.is_field_access());
}

#[test]
fn test_query_field_access_paren() {
    let (offset, src) = src_cursor_offset(
        r#"
            foo.   (id$$ent)
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(q.is_field_access());
}

#[test]
fn test_query_not_field_access() {
    let (offset, src) = src_cursor_offset(
        r#"
            foo.   (a + id$$ent)
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(!q.is_field_access());
}

#[test]
fn test_query_path_middle() {
    let (offset, src) = src_cursor_offset(
        r#"
            p$$ath::foo::bar
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(q.is_path());
}

#[test]
fn test_query_path_before() {
    let (offset, src) = src_cursor_offset(
        r#"
            $$path::foo::bar
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(q.is_path());
}

#[test]
fn test_query_path_after() {
    let (offset, src) = src_cursor_offset(
        r#"
            path::foo::bar$$
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert!(q.is_path());
}

#[test]
fn test_query_path_segment_index() {
    let (offset, src) = src_cursor_offset(
        r#"
            path::foo::bar$$
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert_eq!(q.path_segment_index(), 2);

    let (offset, src) = src_cursor_offset(
        r#"
            path::f$$oo::bar
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert_eq!(q.path_segment_index(), 1);

    let (offset, src) = src_cursor_offset(
        r#"
            $$path::foo::bar
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert_eq!(q.path_segment_index(), 0);

    let (offset, src) = src_cursor_offset(
        r#"
            path$$::foo::bar
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert_eq!(q.path_segment_index(), 0);

    let (offset, src) = src_cursor_offset(
        r#"
            path::foo::$$
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    let q = Query::at(&syntax, offset);

    assert_eq!(q.path_segment_index(), 2);
}

#[test]
fn test_complete_ref() {
    let (offsets, src) = src_cursor_offsets(
        r#"$$
            fn asd() {
                if hello() is b {
                    $$
                } else {
                    $$
                }
            }

            let a =$$ $$;
            $$

            let foo =$$f;

            let b = $$ + $$;

            let c = $$+ b;

            fn a() {
                3*$$
            }

            fn b() {
                a$$
            }

            const bar = $$;

            {$$

                $$
            }$$
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    for (idx, offset) in offsets.enumerate() {
        let q = Query::at(&syntax, offset);
        assert!(q.can_complete_ref(), "test failed for index {idx}",);
    }
}

#[test]
fn test_complete_op() {
    let (offsets, src) = src_cursor_offsets(
        r#"
            a = `${a $$}`;

            let a = 3 $$;

            let b = a $$;

            let a = hm o$$;

            let hm = foo $$o$$p$$;

            let c = "foo" $$

            let in_template = `a ${a $$} b`;
            "#,
    );

    let syntax = Parser::new(&src)
        .with_operator("op", Operator::default())
        .parse_script()
        .into_syntax();

    for (idx, offset) in offsets.enumerate() {
        let q = Query::at(&syntax, offset);
        assert!(q.can_complete_op(), "test failed for index {idx}",);
    }

    let (offsets, src) = src_cursor_offsets(r#"a op 2 $$"#);

    let syntax = Parser::new(&src)
        .with_operator("op", Operator::default())
        .parse_script()
        .into_syntax();

    for (idx, offset) in offsets.enumerate() {
        let q = Query::at(&syntax, offset);
        assert!(q.can_complete_op(), "test failed for index {idx}",);
    }
}

#[test]
fn test_op_completion_should_fail() {
    let (offsets, src) = src_cursor_offsets(
        r#"
        let a = $$ op foo;

        let a = 3* $$;
    
        let a = 3*$$;
    
        let b = $$ + $$;

        fn asd() {
            if hello() + 2 {
                $$
            } else {
                $$
            }
        }
        "#,
    );

    let syntax = Parser::new(&src)
        .with_operator("op", Operator::default())
        .parse_script()
        .into_syntax();

    for (idx, offset) in offsets.enumerate() {
        let q = Query::at(&syntax, offset);
        assert!(!q.can_complete_op(), "test failed for index {idx}",);
    }
}

#[test]
fn test_complete_ref_fails_in_fn_def() {
    let (offsets, src) = src_cursor_offsets(
        r#"
            f$$n$$ $$as$$d$$($$a, b$$)$$ $$ $${
                return 2;
            }
            "#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    for (idx, offset) in offsets.enumerate() {
        let q = Query::at(&syntax, offset);
        assert!(!q.can_complete_ref(), "test failed for index {idx}",);
    }
}

#[test]
fn test_complete_ref_fails_in_decl() {
    let (offsets, src) = src_cursor_offsets(
        r#"
            c$$onst$$ a $$= 2;

            le$$t $$b $$= 3;

            const a $$"#,
    );

    let syntax = Parser::new(&src).parse_script().into_syntax();

    for (idx, offset) in offsets.enumerate() {
        let q = Query::at(&syntax, offset);
        assert!(!q.can_complete_ref(), "test failed for index {idx}",);
    }
}
