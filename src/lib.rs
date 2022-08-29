use bitvec::vec::BitVec;

pub const USAGE: &str = "\
Usage: rust-atomic-primes [options] N\n  \
    where\n    \
        N       (required): an unsigned integer greater than 1, may include _\n    \
        options (optional): any combination of\n      \
            --time (-t): enables timing\n      \
            --all  (-a): prints all primes up to N";

pub fn max_prime(prime_bits: &BitVec) -> Option<usize> {
    // Need to use match or if let until unzip is stabilized
    if let Some((mp, _)) = prime_bits
        .iter()
        .by_vals()
        .enumerate()
        .rev()
        .find(|(_, prime)| *prime)
    {
        Some(mp)
    } else {
        None
    }
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

// pub fn basic_threaded_soe(max: usize) -> BitVec {

// }

#[cfg(test)]
mod tests {
    mod data;
    use crate::{all_primes, max_prime, simple_soe};
    use bitvec::vec::BitVec;
    use data::PRIMES_TO_10_000;

    const SIEVES: [fn(usize) -> BitVec; 1] = [simple_soe];

    fn check(prime_bits: BitVec, mp: Option<usize>, aps: &[usize]) {
        assert_eq!(max_prime(&prime_bits), mp);
        assert_eq!(all_primes(&prime_bits), aps);
    }

    #[test]
    fn simple_soe_10_k() {
        check(
            simple_soe(PrimeData10K::MAX),
            Some(PrimeData10K::MAX_PRIME),
            &PRIMES_TO_10_000,
        );
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
            check(sieve(2), Some(2), &[2]);
        }
    }
}
