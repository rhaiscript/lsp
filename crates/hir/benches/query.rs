use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use itertools::Itertools;
use pprof::criterion::{Output, PProfProfiler};
use rhai_hir::{Hir, Module, Symbol};
use rhai_rowan::parser::Parser;
use std::fs;

fn local_symbols(module: &Module) -> Option<Symbol> {
    let mut last = None;

    for (sym, _) in module.symbols() {
        for visible in module.visible_symbols_from_symbol(sym) {
            last = Some(visible);
        }
    }

    last
}

fn bench(c: &mut Criterion) {
    let modules = fs::read_dir("testdata/benchmarks")
        .unwrap()
        .map(Result::unwrap)
        .map(|entry| {
            (
                entry.path().to_str().unwrap().to_string(),
                fs::read_to_string(entry.path()).unwrap(),
            )
        })
        .map(|(name, src)| {
            let p = Parser::new(&src).parse();
            debug_assert!(p.errors.is_empty());
            (name, p.into_syntax())
        });

    let mut hir = Hir::new();

    for (name, syntax) in modules {
        hir.add_module_from_syntax(&name, &syntax);
    }

    const MIN_SYMBOLS: usize = 10;
    const MIN_SYMBOL_DIFF: usize = 30;

    let mut last_count = 0;
    let modules_by_symbol_count = hir
        .modules()
        .unique_by(|(_, m)| m.symbol_count())
        .sorted_by(|(_, a), (_, b)| a.symbol_count().cmp(&b.symbol_count()))
        .filter(|(_, m)| {
            let keep = (last_count == 0 && m.symbol_count() >= MIN_SYMBOLS)
                || (m.symbol_count() - last_count > MIN_SYMBOL_DIFF);

            if keep {
                last_count = m.symbol_count();
            }
            keep
        });

    let mut group = c.benchmark_group("visible symbols by symbol count");
    for (_, module) in modules_by_symbol_count {
        let count = module.symbol_count() as u64;

        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::from_parameter(count), module, |b, module| {
            b.iter(|| black_box(local_symbols(module)));
        });
    }
    group.finish();

    const MIN_SCOPE_DIFF: usize = 3;

    let mut last_count = 0;
    let modules_by_scope_count = hir
        .modules()
        .unique_by(|(_, m)| m.scope_count())
        .sorted_by(|(_, a), (_, b)| a.scope_count().cmp(&b.scope_count()))
        .filter(|(_, m)| {
            let keep = last_count == 0
                || (m.scope_count() - last_count > MIN_SCOPE_DIFF);

            if keep {
                last_count = m.scope_count();
            }
            keep
        });

    let mut group = c.benchmark_group("visible symbols by scope count");
    for (_, module) in modules_by_scope_count {
        let count = module.scope_count() as u64;
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::from_parameter(count), module, |b, module| {
            b.iter(|| black_box(local_symbols(module)));
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench
);
criterion_main!(benches);
