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
        error!("{}", rust_atomic_primes::USAGE);
    }
    let max_arg = args.last().unwrap();
    let max = max_arg
        .replace('_', "")
        .parse::<usize>()
        .unwrap_or_else(|err| {
            error!(
                "N must be an unsigned integer. Recieved: {}\n\nParseIntError:\n\t{}",
                max_arg, err
            );
        });
    if max <= 1 {
        error!("N must be greater than 1. Recieved: {}", max);
    }

    // Parse args for options
    let options = &args[1..args.len() - 1];
    let mut time = false;
    let mut all = false;
    for option in options.iter() {
        match option {
            _ if option == "--time" || option == "-t" => time = true,
            _ if option == "--all" || option == "-a" => all = true,
            _ => {
                error!("Invalid option: {}", option);
            }
        }
    }

    // Start timing if needed
    let maybe_start = if time {
        Some(std::time::Instant::now())
    } else {
        None
    };

    // Run
    let prime_bits = rust_atomic_primes::simple_seive_of_eratosthenes(max);

    // Print info

    if let Some(start) = maybe_start {
        println!("Runtime: {:?}", start.elapsed());
    }

    if all {
        let prime_numbers = prime_bits.iter().by_refs().enumerate().fold(
            Vec::new(),
            |mut p_nums_accum, (num, prime)| {
                if *prime {
                    p_nums_accum.push(num);
                }
                p_nums_accum
            },
        );

        println!("Primes less than or equal to {}: {:?}", max, prime_numbers);
    } else {
        let (max_prime, _) = prime_bits
            .iter()
            .by_vals()
            .enumerate()
            .rev()
            .find(|(_, prime)| *prime)
            .unwrap();
        println!("Largest prime less than or equal to {}: {}", max, max_prime);
    }
}
