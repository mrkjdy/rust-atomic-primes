use bitvec::vec::BitVec;

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        println!($($arg)+);
        std::process::exit(1);
    };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse args for the max
    if args.len() < 2 {
        error!("Usage:\trust-atomic-primes max [options]\nwhere\n\tmax is an unsigned integer greater than 1\n\toptions are:\n\t\ttime - to time the seive\n\t\tall - to print all instead of just the max");
    }
    let max = match args[1].replace(',', "").parse::<usize>() {
        Ok(n) => n,
        Err(err) => {
            error!("N must be an unsigned integer\n\nRecieved error:\n\t{}", err);
        }
    };
    if max <= 1 {
        error!("Max must be greater than 1. Recieved: {}", max);
    }

    // Parse args for options
    let options = &args[2..];
    let mut time = false;
    let mut all = false;
    for option in options.iter() {
        match option {
            _ if option == "time" => time = true,
            _ if option == "all" => all = true,
            _ => {
                error!("Invalid option: {}", option);
            }
        }
    }

    let now_option = if time {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let mut prime_bits: BitVec = BitVec::repeat(true, max + 1);
    let len = prime_bits.len();

    prime_bits.set(0, false);
    prime_bits.set(1, false);

    for num in 2..=(len as f64).sqrt() as usize {
        if prime_bits[num] {
            'mul: for factor in num .. {
                let product = num * factor;
                if product >= len {
                    break 'mul;
                }
                prime_bits.set(product, false);
            }
        }
    }

    if let Some(now) = now_option {
        println!("Runtime: {:?}", now.elapsed());
    }
    
    if all {
        let prime_numbers = prime_bits
            .iter()
            .by_refs()
            .enumerate()
            .fold(Vec::new(), | mut p_nums_accum, (num, prime) | {
                if *prime {
                    p_nums_accum.push(num);
                }
                p_nums_accum
            });
    
        println!("Primes less than or equal to {}: {:?}", max, prime_numbers);
    } else {
        let (max_prime, _) = prime_bits
            .iter()
            .by_vals()
            .enumerate()
            .rev()
            .find(| (_, prime) | *prime)
            .unwrap();
        println!("Largest prime less than or equal to {}: {}", max, max_prime);
    }
}
