use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use pprof::criterion::{Output, PProfProfiler};
use rhai_rowan::parser::{Parse, Parser};

fn parse(src: &str) -> Parse {
    let mut parser = Parser::new(src);
    parser.parse_file();
    parser.finish()
}

fn bench(c: &mut Criterion) {
    const SIMPLE_SRC: &str = include_str!("../../../testdata/valid/oop.rhai");
    const OOP_SRC: &str = include_str!("../../../testdata/valid/oop.rhai");

    let mut g = c.benchmark_group("simple");
    g.throughput(Throughput::Bytes(SIMPLE_SRC.as_bytes().len() as u64))
        .bench_function("parse simple", |b| b.iter(|| parse(black_box(SIMPLE_SRC))));
    g.finish();

    let mut g = c.benchmark_group("oop");
    g.throughput(Throughput::Bytes(OOP_SRC.as_bytes().len() as u64))
        .bench_function("parse oop", |b| b.iter(|| parse(black_box(OOP_SRC))));
    g.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench
);
criterion_main!(benches);
