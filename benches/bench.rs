use std::{fmt::Display, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use rust_atomic_primes::{basic_threaded_soe, simple_soe};

struct MaxThreadStruct {
    max: usize,
    thread_count: u8,
}

impl Display for MaxThreadStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}T", self.max, self.thread_count)
    }
}

fn bench(c: &mut Criterion) {
    let max_size_dur_tuples = [
        (1_000, 100, 5),
        (100_000, 100, 5),
        (10_000_000, 20, 5),
        (1_000_000_000, 10, 100),
    ];
    let thread_counts = [1, 2, 3, 4, 8];
    for (max, size, dur) in max_size_dur_tuples {
        let mut group = c.benchmark_group(format!("max {}", max));
        group
            .sampling_mode(SamplingMode::Flat)
            .sample_size(size)
            .measurement_time(Duration::from_secs(dur));
        group.bench_with_input(BenchmarkId::new("simple_soe", max), &max, |b, &m| {
            b.iter(|| simple_soe(m))
        });
        for thread_count in thread_counts {
            let max_thread_struct = MaxThreadStruct { max, thread_count };
            group.bench_with_input(
                BenchmarkId::new("basic_threaded_soe", &max_thread_struct),
                &max_thread_struct,
                |b, mts| b.iter(|| basic_threaded_soe(mts.max, mts.thread_count)),
            );
        }
        group.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
