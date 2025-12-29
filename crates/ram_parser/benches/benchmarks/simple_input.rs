use std::path::PathBuf;

use codspeed_criterion_compat::{BenchmarkId, Criterion, Throughput, criterion_group};

fn parse(c: &mut Criterion) {
    let crate_root = env!("CARGO_MANIFEST_DIR");

    let path = PathBuf::from(crate_root).join("../../inputs");

    let mut group = c.benchmark_group("parse");

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_str().unwrap();
            let source = std::fs::read_to_string(&path).unwrap();
            group.throughput(Throughput::Bytes(source.len() as u64));
            group.bench_with_input(BenchmarkId::from_parameter(name), &source, |b, source| {
                b.iter(|| ram_parser::parse(source));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, parse);
