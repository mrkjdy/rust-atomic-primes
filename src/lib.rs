use bitvec::vec::BitVec;

pub const USAGE: &str = "Usage: rust-atomic-primes [options] N\n  \
    where\n    \
        N       (required): an unsigned integer greater than 1, may include _\n    \
        options (optional): any combination of\n      \
            --time (-t): enables timing\n      \
            --all  (-a): prints all primes up to N";

pub fn max_prime(prime_bits: &BitVec) -> usize {
    prime_bits
        .iter()
        .by_vals()
        .enumerate()
        .rev()
        .find(|(_, prime)| *prime)
        .unwrap()
        .0
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

    prime_bits.set(0, false);
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
    use data::PrimeData10K;

    #[test]
    fn simple_soe_test() {
        let prime_bits = simple_soe(PrimeData10K::MAX);
        assert_eq!(prime_bits.len(), PrimeData10K::MAX + 1);
        assert_eq!(max_prime(&prime_bits), PrimeData10K::MAX_PRIME);
        assert_eq!(all_primes(&prime_bits), PrimeData10K::ALL_PRIMES);
    }
}
