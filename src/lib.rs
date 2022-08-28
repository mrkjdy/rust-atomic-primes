use bitvec::vec::BitVec;

pub const USAGE: &str = "Usage: rust-atomic-primes [options] N\n  \
    where\n    \
        N       (required): an unsigned integer greater than 1, may include _\n    \
        options (optional): any combination of\n      \
            --time (-t): enables timing\n      \
            --all  (-a): prints all primes up to N";

pub fn simple_seive_of_eratosthenes(max: usize) -> BitVec {
    let mut prime_bits: BitVec = BitVec::repeat(true, max + 1);
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
