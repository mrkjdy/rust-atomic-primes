use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use rust_atomic_primes::{basic_threaded_soe, cache_sized_soe, simple_soe};

fn sieve_bench(c: &mut Criterion) {
    let max_size_dur_tuples = [
        (1_000, 100, 5),
        (100_000, 100, 5),
        (10_000_000, 20, 5),
        (1_000_000_000, 10, 100),
    ];
    let sizes_kb: Vec<usize> = (0..=20).map(|exp| 2_usize.pow(exp)).collect();
    let thread_counts = [1, 2, 3, 4, 8];
    for (max, size, dur) in max_size_dur_tuples {
        let mut group = c.benchmark_group(format!("max {}", max));
        group
            .sampling_mode(SamplingMode::Flat)
            .sample_size(size)
            .measurement_time(Duration::from_secs(dur));
        group.bench_with_input("simple_soe", &max, |b, &m| b.iter(|| simple_soe(m)));
        for thread_count in thread_counts {
            group.bench_with_input(
                BenchmarkId::new("basic_threaded_soe", format!("{}T", thread_count)),
                &(max, thread_count),
                |b, &(m, t)| b.iter(|| basic_threaded_soe(m, t)),
            );
        }
        for size_kb in sizes_kb.iter() {
            group.bench_with_input(
                BenchmarkId::new("cache_sized_soe", format!("{}KB", size_kb)),
                &(max, size_kb),
                |b, &(m, &kb)| b.iter(|| cache_sized_soe(m, kb * 1024)),
            );
        }
        group.finish();
    }
}

criterion_group!(benches, sieve_bench);
criterion_main!(benches);
