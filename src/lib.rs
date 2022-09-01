use bitvec::vec::BitVec;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub const USAGE: &str = "\
Usage: rust-atomic-primes [options] N\n  \
    where\n    \
        N       (required): an unsigned integer greater than 1, may include _\n    \
        options (optional): any combination of\n      \
            --time (-t): enables timing\n      \
            --all  (-a): prints all primes up to N";

pub fn max_prime(prime_bits: &BitVec) -> Option<usize> {
    prime_bits
        .iter()
        .by_vals()
        .enumerate()
        .rev()
        .find(|(_, prime)| *prime)
        .map(|(mp, _)| mp)
}

pub fn all_primes(prime_bits: &BitVec) -> Vec<usize> {
    prime_bits
        .iter()
        .by_refs()
        .enumerate()
        .fold(Vec::new(), |mut p_nums_accum, (num, prime)| {
            if *prime {
                p_nums_accum.push(num);
            }
            p_nums_accum
        })
}

pub fn simple_soe(max: usize) -> BitVec {
    let mut prime_bits = BitVec::repeat(true, max + 1);
    let len = prime_bits.len();

    // Zero is not prime
    prime_bits.set(0, false);

    if len < 2 {
        return prime_bits;
    }

    // One is not prime
    prime_bits.set(1, false);

    for num in 2..=(len as f64).sqrt() as usize {
        if prime_bits[num] {
            'mul: for factor in num.. {
                let product = num * factor;
                if product >= len {
                    break 'mul;
                }
                prime_bits.set(product, false);
            }
        }
    }

    prime_bits
}

fn usize_div_ceil(numerator: usize, denominator: usize) -> usize {
    (numerator + denominator - 1) / denominator
}

fn basic_soe_thread(
    current_prime_rx: Receiver<usize>,
    first_true_tx: Sender<Option<usize>>,
    start: usize,
    size: usize,
) -> BitVec {
    let mut prime_bits = BitVec::repeat(true, size);
    if start == 0 {
        prime_bits.set(0, false);
        if size > 1 {
            prime_bits.set(1, false);
        }
    }
    let max_val = start + size - 1;
    let mut current_prime = 0;
    let mut index = 0;
    loop {
        // Send the main thread number associated with first true in bit array
        let mut maybe_first_true = None;
        'find: while index < size {
            if prime_bits[index] {
                let number = index + start;
                if number > current_prime {
                    maybe_first_true = Some(number);
                    break 'find;
                }
            }
            index += 1;
        }
        first_true_tx.send(maybe_first_true).unwrap();
        // Receive the prime to mark
        current_prime = match current_prime_rx.recv() {
            Ok(prime) => prime,
            Err(_) => break,
        };
        // Mark each multiple of the current prime in this threads range as not prime
        let first_factor = usize_div_ceil(start, current_prime);
        'mark: for factor in first_factor.. {
            let product = current_prime * factor;
            if product > max_val {
                break 'mark;
            }
            if product > current_prime {
                prime_bits.set(product - start, false);
            }
        }
    }
    prime_bits
}

struct SoEThread {
    // id: u8,
    current_prime_tx: Sender<usize>,
    first_true_rx: Receiver<Option<usize>>,
    handle: JoinHandle<BitVec>,
}

pub fn basic_threaded_soe(max: usize) -> BitVec {
    let num_threads = 1;
    let len = max + 1;
    let remainder = usize::from(len % (num_threads as usize) > 0);
    let chunk_size = len / num_threads as usize;
    let threads: Vec<SoEThread> = (0..num_threads)
        .map(|id| {
            let (current_prime_tx, current_prime_rx) = mpsc::channel();
            let (first_true_tx, first_true_rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                basic_soe_thread(
                    current_prime_rx,
                    first_true_tx,
                    chunk_size * id as usize,
                    match id {
                        0 => chunk_size + remainder,
                        _ => chunk_size,
                    },
                )
            });
            SoEThread {
                current_prime_tx,
                first_true_rx,
                handle,
            }
        })
        .collect();
    let largest_root = (len as f64).sqrt() as usize;
    loop {
        // Recieve the first true from each thread and find the lowest
        let mut lowest_true = usize::MAX;
        for thread in threads.iter() {
            let maybe_first_true = thread.first_true_rx.recv().unwrap();
            if let Some(first_true) = maybe_first_true {
                if first_true < lowest_true {
                    lowest_true = first_true;
                }
            }
        }
        // Don't need to mark larger primes because their multiples have already been marked by the
        // lower primes.
        if lowest_true > largest_root {
            break;
        }
        // Send the threads the first prime to mark
        for thread in threads.iter() {
            thread.current_prime_tx.send(lowest_true).unwrap();
        }
    }
    let mut all_bits = BitVec::new();
    for thread in threads {
        drop(thread.current_prime_tx);
        all_bits.append(&mut thread.handle.join().unwrap());
    }
    all_bits
}

#[cfg(test)]
mod tests {
    mod data;
    use crate::{all_primes, basic_threaded_soe, max_prime, simple_soe};
    use bitvec::vec::BitVec;
    use data::PRIMES_TO_10_000;

    const SIEVES: [fn(usize) -> BitVec; 2] = [simple_soe, basic_threaded_soe];

    fn check(prime_bits: &BitVec, mp: Option<usize>, aps: &[usize]) {
        assert_eq!(max_prime(prime_bits), mp);
        assert_eq!(all_primes(prime_bits), aps);
    }

    #[test]
    fn simple_soe_10_k() {
        check(&simple_soe(10_000), Some(9_973), &PRIMES_TO_10_000)
    }

    #[test]
    fn all_0() {
        for sieve in SIEVES {
            check(&sieve(0), None, &[]);
        }
    }

    #[test]
    fn all_1() {
        for sieve in SIEVES {
            check(&sieve(1), None, &[]);
        }
    }

    #[test]
    fn all_2() {
        for sieve in SIEVES {
            check(&sieve(2), Some(2), &[2]);
        }
    }

    #[test]
    fn all_10() {
        const MAX: usize = 10;
        for sieve in SIEVES {
            check(&sieve(MAX), Some(7), &[2, 3, 5, 7]);
        }
    }

    #[test]
    fn all_1_000_000() {
        const MAX: usize = 1_000_000;
        let expected_prime_bits = simple_soe(MAX);
        let expected_max_prime = max_prime(&expected_prime_bits);
        let expected_all_primes = all_primes(&expected_prime_bits);
        for sieve in SIEVES {
            check(&sieve(MAX), expected_max_prime, &expected_all_primes)
        }
    }
}
