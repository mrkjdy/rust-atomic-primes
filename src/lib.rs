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

pub fn simple_soe(max: usize) -> Vec<usize> {
    let mut prime_bits = BitVec::repeat(true, max + 1);
    let len = prime_bits.len();

    // Zero is not prime
    prime_bits.set(0, false);

    if len < 2 {
        return all_primes(&prime_bits);
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

    all_primes(&prime_bits)
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
    }
    if 1 >= start {
        let one_index = 1 - start;
        if one_index < size {
            prime_bits.set(one_index, false);
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

pub fn basic_threaded_soe(max: usize, num_threads: u8) -> Vec<usize> {
    let len = max + 1;
    let remainder = len % (num_threads as usize);
    let chunk_size = len / num_threads as usize;
    let threads: Vec<SoEThread> = (0..num_threads)
        .map(|id| {
            let (current_prime_tx, current_prime_rx) = mpsc::channel();
            let (first_true_tx, first_true_rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                let start;
                let size;
                if id == 0 {
                    start = 0;
                    size = chunk_size + remainder;
                } else {
                    start = chunk_size * id as usize + remainder;
                    size = chunk_size;
                }
                basic_soe_thread(current_prime_rx, first_true_tx, start, size)
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
    all_primes(&all_bits)
}

// fn cache_sized_soe(max: usize) -> Vec<usize> {
// good_cache_size = find good cache size
// checked_up_to = 0
// create uninitialized bitvec with size good_cache_size min(page size, max)
// create list of primes to mark
// 'block: for each good_cache_sized range of numbers up to max
//   initialize the bitvec (current block) to all true
//   for each existing prime to mark
//     mark all multiples within this block up to sqrt max
//   for each number in checked_up_to through sqrt max
//     if the current number is not in the current block
//       break 'block;
//     if the current number is still true (thus prime)
//       add the current number to the list of primes to mark
//       mark all multiples within this block up to sqrt max
//     increment checked_up_to
//   for each number in checked_up_to through n
//     if the current number is not in the current block
//       break 'block;
//     if the current number is still true (thus prime)
//       add the current number to the list of primes that don't need to be marked
// return primes_to_mark.concat(other primes)
// }

#[cfg(test)]
mod tests {
    mod data;
    use crate::{basic_threaded_soe, simple_soe};
    use data::PRIMES_TO_10_000;

    const SIEVES: [fn(usize) -> Vec<usize>; 6] = [
        simple_soe,
        |max: usize| basic_threaded_soe(max, 1),
        |max: usize| basic_threaded_soe(max, 2),
        |max: usize| basic_threaded_soe(max, 3),
        |max: usize| basic_threaded_soe(max, 4),
        |max: usize| basic_threaded_soe(max, 10),
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
    fn all_1_000_000() {
        const MAX: usize = 1_000_000;
        let expected_primes = simple_soe(MAX);
        let expected_max_prime = expected_primes.last();
        for sieve in SIEVES {
            check(sieve(MAX), expected_max_prime, &expected_primes)
        }
    }
}
