use anyhow::{Result, anyhow};
use rayon::prelude::*;
use std::time::Instant;
use util::io::{display_menu, ninput, pause};

fn main() {
    let items = vec!["Sum", "Is prime", "Sum of prime numbers", "Exit"];

    loop {
        let choice: usize = display_menu(&items, None).unwrap_or_else(|ex| {
            eprintln!("{}", ex);
            10
        });
        let result = match choice {
            1 => do_sum(),
            2 => do_prime(),
            3 => do_sum_prime(),
            _ => {
                if choice == 0 {
                    println!("Exiting the application.");
                    std::process::exit(0);
                }

                Err(anyhow!("Invalid option. Please try again."))
            }
        };

        if let Err(ex) = result {
            eprintln!("{}", ex);
            pause();
        }
    }
}

fn do_sum() -> Result<()> {
    let input: u64 = ninput(Some("Enter a number (Leave empty to exit): ")).unwrap_or(0);

    if input < 1 {
        println!("Sum: 0. took 0 seconds");
        return Ok(());
    }

    let start = Instant::now();
    let sum = (0..=input).into_par_iter().sum::<u64>();
    let ellapsed = start.elapsed();
    println!("Sum: {sum}. took {} seconds", ellapsed.as_secs_f64());
    pause();
    Ok(())
}

fn do_prime() -> Result<()> {
    let input: u64 = ninput(Some("Enter a number (Leave empty to exit): ")).unwrap_or(0);

    if input < 2 {
        println!("{input} is not prime. took 0 seconds");
        return Ok(());
    }

    let start = Instant::now();
    let result = is_prime(input);
    let ellapsed = start.elapsed();
    println!(
        "{} {} prime. took {} time",
        input,
        if result { "is" } else { "is not" },
        ellapsed.as_secs_f64()
    );
    pause();
    Ok(())
}

fn do_sum_prime() -> Result<()> {
    let input: u64 = ninput(Some("Enter a number (Leave empty to exit): ")).unwrap_or(0);

    if input < 1 {
        println!("Sum: 0. took 0 seconds");
        return Ok(());
    }

    let start = Instant::now();
    let sum = (0..=input)
        .into_par_iter()
        .filter(|x| is_prime(*x))
        .sum::<u64>();
    let ellapsed = start.elapsed();
    println!(
        "Sum of prime numbers between 0 and {input}: {sum}. took {} seconds",
        ellapsed.as_secs_f64()
    );
    pause();
    Ok(())
}

fn is_prime(n: u64) -> bool {
    (2..=n / 2).into_par_iter().all(|x| n % x != 0)
}
