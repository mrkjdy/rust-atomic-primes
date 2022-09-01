use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use rust_atomic_primes::{basic_threaded_soe, simple_soe};

fn bench_1_000(c: &mut Criterion) {
    let max = 1_000;
    let mut group = c.benchmark_group(format!("max-{}", max));
    group.sample_size(50);
    group.bench_function(&format!("simple_soe({})", max), |b| {
        b.iter(|| simple_soe(max))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 1)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 1))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 2)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 2))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 3)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 3))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 4)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 4))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 10)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 10))
    });
    group.finish();
}

fn bench_1_000_000(c: &mut Criterion) {
    let max = 1_000_000;
    let mut group = c.benchmark_group(format!("max-{}", max));
    group.sample_size(50);
    group.bench_function(&format!("simple_soe({})", max), |b| {
        b.iter(|| simple_soe(max))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 1)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 1))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 2)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 2))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 3)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 3))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 4)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 4))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 10)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 10))
    });
    group.finish();
}

fn bench_100_000_000(c: &mut Criterion) {
    let max = 100_000_000;
    let mut group = c.benchmark_group(format!("max-{}", max));
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(12));
    group.bench_function(&format!("simple_soe({})", max), |b| {
        b.iter(|| simple_soe(max))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 1)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 1))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 2)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 2))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 3)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 3))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 4)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 4))
    });
    group.bench_function(&format!("basic_threaded_soe({}, 10)", max), |b| {
        b.iter(|| basic_threaded_soe(max, 10))
    });
    group.finish();
}

criterion_group!(benches, bench_1_000, bench_1_000_000, bench_100_000_000);
criterion_main!(benches);
