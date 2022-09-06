use bitvec::vec::BitVec;
// use cache_size::l1_cache_size;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub const USAGE: &str = "\
Usage: rust-atomic-primes [options] N\n  \
    where\n    \
        N       (required): an unsigned integer greater than 1, may include _\n    \
        options (optional): any combination of\n      \
            --time (-t): enables timing\n      \
            --all  (-a): prints all primes up to N";

pub fn simple_soe(max: usize) -> Vec<usize> {
    let mut bits: BitVec = BitVec::repeat(true, max + 1);
    let mut primes = Vec::new();
    let mut num = 2;
    // Mark the primes up to the sqrt of max
    while num <= (max as f64).sqrt() as usize {
        if bits[num] {
            primes.push(num);
            let mut multiple = num * num;
            while multiple <= max {
                bits.set(multiple, false);
                multiple += num;
            }
        }
        num += 1;
    }
    // Collect the remaining primes
    while num <= max {
        if bits[num] {
            primes.push(num);
        }
        num += 1;
    }
    primes
}

fn usize_div_ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

fn basic_soe_thread(
    current_prime_rx: Receiver<usize>,
    first_true_tx: Sender<Option<usize>>,
    max: usize,
    offset: usize,
    capacity: usize,
) -> Vec<usize> {
    let mut bits: BitVec = BitVec::repeat(true, capacity);
    let max_val = offset + capacity - 1;
    let mut num = usize::max(offset, 2);
    loop {
        // Send the main thread number associated with first true in bit array
        let mut maybe_first_true = None;
        let mut sent = 0;
        while num <= (max as f64).sqrt() as usize && num <= max_val {
            if bits[num - offset] {
                maybe_first_true = Some(num);
                // Can't increment yet, need to see if it actually gets used
                sent = num;
                break;
            }
            num += 1;
        }
        first_true_tx.send(maybe_first_true).unwrap();
        // Receive a prime to mark
        match current_prime_rx.recv() {
            // Exit if the main thread drops this channel
            Err(_) => break,
            // Otherwise mark all the primes
            Ok(prime) => {
                // Increment if the value sent was the value that got used
                if sent == prime {
                    num += 1;
                }
                mark_multiples(&mut bits, prime, max_val, offset);
            }
        };
    }
    // Send the main thread the remaining primes
    let mut other_primes = Vec::new();
    while num <= max_val {
        if bits[num - offset] {
            other_primes.push(num);
        }
        num += 1;
    }
    other_primes
}

struct SoEThread {
    current_prime_tx: Sender<usize>,
    first_true_rx: Receiver<Option<usize>>,
    handle: JoinHandle<Vec<usize>>,
}

pub fn basic_threaded_soe(max: usize, num_threads: u8) -> Vec<usize> {
    let len = max + 1;
    let remainder = len % (num_threads as usize);
    let chunk_size = len / num_threads as usize;
    let threads: Vec<SoEThread> = (0..num_threads)
        .map(|id| {
            let (current_prime_tx, current_prime_rx) = mpsc::channel();
            let (first_true_tx, first_true_rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                if id == 0 {
                    basic_soe_thread(
                        current_prime_rx,
                        first_true_tx,
                        max,
                        0,
                        chunk_size + remainder,
                    )
                } else {
                    basic_soe_thread(
                        current_prime_rx,
                        first_true_tx,
                        max,
                        chunk_size * id as usize + remainder,
                        chunk_size,
                    )
                }
            });
            SoEThread {
                current_prime_tx,
                first_true_rx,
                handle,
            }
        })
        .collect();
    let mut primes = Vec::new();
    loop {
        // Recieve the first true from each thread and find the lowest prime candidate
        let mut maybe_lowest_true = None;
        let mut current_lowest = usize::MAX;
        for thread in threads.iter() {
            let maybe_first_true = thread.first_true_rx.recv().unwrap();
            if let Some(first_true) = maybe_first_true {
                if first_true < current_lowest {
                    current_lowest = first_true;
                    maybe_lowest_true = maybe_first_true;
                }
            }
        }
        // Send the threads the current prime to mark or break
        match maybe_lowest_true {
            // Break if no prime candidates were found
            None => break,
            // Otherwise send each thread a prime to mark
            Some(prime_to_mark) => {
                for thread in threads.iter() {
                    thread.current_prime_tx.send(prime_to_mark).unwrap();
                }
                primes.push(prime_to_mark);
            }
        }
    }
    // Collect the remaining primes
    for thread in threads {
        drop(thread.current_prime_tx);
        primes.append(&mut thread.handle.join().unwrap());
    }
    primes
}

fn mark_multiples(bits: &mut BitVec, number: usize, max: usize, offset: usize) {
    let mut multiple = number * usize::max(number, usize_div_ceil(offset, number));
    while multiple <= max {
        bits.set(multiple - offset, false);
        multiple += number;
    }
}

pub fn cache_sized_soe(max: usize, bitvec_size: usize) -> Vec<usize> {
    let fast_capacity = bitvec_size * 8;
    let capacity = usize::min(max + 1, fast_capacity);
    let sqrt_of_max = (max as f64).sqrt() as usize;
    let mut bits: BitVec = BitVec::with_capacity(capacity);
    unsafe {
        bits.set_len(capacity);
    }
    // let mut primes_to_mark = Vec::new();
    // let mut all_primes = Vec::new();
    let mut primes = Vec::new();
    let mut num = 2;
    let mut offset = 0;
    'chunk: while offset <= max {
        bits.fill(true);
        let chunk_max = usize::min(offset + capacity - 1, max);
        for &prime in primes.iter() {
            if prime > sqrt_of_max {
                break;
            }
            mark_multiples(&mut bits, prime, chunk_max, offset);
        }
        while num <= sqrt_of_max {
            if num > chunk_max {
                offset += capacity;
                continue 'chunk;
            }
            if num >= offset && bits[num - offset] {
                primes.push(num);
                mark_multiples(&mut bits, num, chunk_max, offset);
            }
            num += 1;
        }
        while num <= max {
            if num > chunk_max {
                offset += capacity;
                continue 'chunk;
            }
            if num >= offset && bits[num - offset] {
                primes.push(num);
            }
            num += 1;
        }
        offset += capacity;
    }
    return primes;
}

#[cfg(test)]
mod tests {
    mod data;
    use crate::{basic_threaded_soe, cache_sized_soe, simple_soe};
    use data::PRIMES_TO_10_000;

    const SIEVES: [fn(usize) -> Vec<usize>; 7] = [
        simple_soe,
        |max: usize| basic_threaded_soe(max, 1),
        |max: usize| basic_threaded_soe(max, 2),
        |max: usize| basic_threaded_soe(max, 3),
        |max: usize| basic_threaded_soe(max, 4),
        |max: usize| basic_threaded_soe(max, 10),
        |max: usize| cache_sized_soe(max, 32768),
    ];

    fn check(
        primes: Vec<usize>,
        expected_largest_prime: Option<&usize>,
        expected_primes: &[usize],
    ) {
        assert_eq!(primes.last(), expected_largest_prime);
        assert_eq!(primes, expected_primes);
    }

    #[test]
    fn simple_soe_10_k() {
        check(simple_soe(10_000), Some(&9_973), &PRIMES_TO_10_000)
    }

    #[test]
    fn all_0() {
        for sieve in SIEVES {
            check(sieve(0), None, &[]);
        }
    }

    #[test]
    fn all_1() {
        for sieve in SIEVES {
            check(sieve(1), None, &[]);
        }
    }

    #[test]
    fn all_2() {
        for sieve in SIEVES {
            check(sieve(2), Some(&2), &[2]);
        }
    }

    #[test]
    fn all_10() {
        for sieve in SIEVES {
            check(sieve(10), Some(&7), &[2, 3, 5, 7]);
        }
    }

    #[test]
    fn all_100() {
        const MAX: usize = 100;
        let expected_primes = simple_soe(MAX);
        let expected_max_prime = expected_primes.last();
        for sieve in SIEVES {
            check(sieve(MAX), expected_max_prime, &expected_primes);
        }
    }

    #[test]
    fn all_1_000_000() {
        const MAX: usize = 1_000_000;
        let expected_primes = simple_soe(MAX);
        let expected_max_prime = expected_primes.last();
        for sieve in SIEVES {
            check(sieve(MAX), expected_max_prime, &expected_primes)
        }
    }
}
