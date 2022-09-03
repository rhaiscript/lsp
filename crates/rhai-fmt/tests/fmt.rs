use pretty_assertions::assert_eq;

macro_rules! assert_fmt {
    ($src:expr) => {
        {
            let s = $src;
            assert_eq!(s, rhai_fmt::format_source(s, Default::default()));
        }
    };
    ($src:expr, $expected:expr) => {
        {
            let s = $src;
            assert_eq!($expected, rhai_fmt::format_source(s));
        }
    };
}

#[test]
fn fmt_smoke() {
    let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).try_init();

    assert_fmt!("/// hello
/// there
let a = 1234;
let b = 'a'
");
}
