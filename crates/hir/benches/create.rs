use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use rhai_hir::Hir;
use rhai_rowan::{parser::Parser, syntax::SyntaxNode};

fn create_hir(syntax: &SyntaxNode) -> Hir {
    let mut hir = Hir::new();
    hir.add_module_from_syntax("bench", &syntax);
    hir
}

fn create_hir_full(syntax: &SyntaxNode) -> Hir {
    let mut hir = Hir::new();
    hir.add_module_from_syntax("bench", &syntax);
    hir.resolve_references();
    hir
}

fn bench(c: &mut Criterion) {
    const SIMPLE_SRC: &str = include_str!("../../../testdata/valid/simple.rhai");
    const OOP_SRC: &str = include_str!("../../../testdata/valid/oop.rhai");

    let simple_parse = Parser::new(SIMPLE_SRC).parse();
    debug_assert!(simple_parse.errors.is_empty());

    let oop_parse = Parser::new(OOP_SRC).parse();
    debug_assert!(oop_parse.errors.is_empty());

    c.bench_function("simple hir create", |b| {
        b.iter(|| create_hir(black_box(&simple_parse.clone_syntax())))
    });
    c.bench_function("simple hir full", |b| {
        b.iter(|| create_hir_full(black_box(&simple_parse.clone_syntax())))
    });
    c.bench_function("oop hir create", |b| {
        b.iter(|| create_hir(black_box(&oop_parse.clone_syntax())))
    });
    c.bench_function("oop hir full", |b| {
        b.iter(|| create_hir_full(black_box(&oop_parse.clone_syntax())))
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench
);
criterion_main!(benches);
