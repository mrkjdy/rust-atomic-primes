use std::{fmt::Display, mem::size_of, time::Duration};

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

fn sieve_bench(c: &mut Criterion) {
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

fn cache_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache performance");
    group.measurement_time(Duration::from_secs(10));
    let exponents = 0..=20;
    let mask_size_kb_pairs: Vec<(usize, usize)> = exponents
        .map(|exponent| (!(usize::MAX << exponent + 3), 2_usize.pow(exponent)))
        .collect();
    for (mask, size_kb) in mask_size_kb_pairs {
        let capacity = size_kb * (1024 / size_of::<usize>());
        group.bench_with_input(
            BenchmarkId::new("access_sim", size_kb),
            &(capacity, mask),
            |b, &(c, m)| {
                let mut arr: Vec<usize> = Vec::with_capacity(c);
                unsafe {
                    arr.set_len(c);
                }
                b.iter(|| {
                    const MAX: usize = 1_000_000;
                    let mut sum = 0;
                    let mut loc = 0;
                    let mut del = 0;
                    for _ in 0..MAX {
                        let masked_loc = m & loc;
                        arr[masked_loc] += 1;
                        sum += arr[masked_loc];
                        loc += del;
                        del += sum;
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, sieve_bench, cache_bench);
criterion_main!(benches);
