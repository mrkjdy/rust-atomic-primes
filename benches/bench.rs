use criterion::{criterion_group, criterion_main, Criterion};
use rust_atomic_primes::simple_soe;

const MAX: usize = 1_000_000;

fn simple_soe_benchmark(c: &mut Criterion) {
    c.bench_function(&format!("simple_soe({})", MAX), |b| {
        b.iter(|| crate::simple_soe(MAX))
    });
}

criterion_group!(benches, simple_soe_benchmark);
criterion_main!(benches);
