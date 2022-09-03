use std::{ffi::OsStr, fs, path::Path};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rhai_fmt::{format_source, format_syntax};

fn mega_script() -> String {
    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("testdata")
        .join("valid");
    let scripts = fs::read_dir(&root_path).unwrap();

    let mut script = String::new();

    for entry in scripts {
        let entry = entry.unwrap();

        if entry.path().extension() == Some(OsStr::new("rhai")) {
            script += &fs::read_to_string(root_path.join(entry.path())).unwrap();
            script += ";\n";
        }
    }

    script
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let source = mega_script();

    let parsed = rhai_rowan::parser::Parser::new(&source)
        .parse_script()
        .into_syntax();

    let mut group = c.benchmark_group("fmt-throughput");
    group.throughput(Throughput::Bytes(source.len() as u64));
    group.bench_function("fmt all", |b| {
        b.iter(|| format_source(black_box(&source), Default::default()))
    });
    group.bench_function("fmt parsed", |b| {
        b.iter(|| format_syntax(black_box(parsed.clone()), Default::default()))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
