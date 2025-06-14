use util::ninput;

fn parkable(n: u32) {
    loop {
        std::thread::park();
        println!(">>> Thread {n} was unparked briefly.");
    }
}

fn main() {
    let mut threads = vec![];

    for i in 0..10 {
        threads.push(std::thread::spawn(move || {
            parkable(i);
        }));
    }

    loop {
        let input = ninput(Some("Enter a number to unpark a thread (0 to exit): ")).unwrap_or(0);

        if input == 0 {
            break;
        } else if input < threads.len() as u32 {
            println!("Unparking thread {input}.");
            threads[input as usize].thread().unpark();
        } else {
            println!("Invalid thread number: {input}. Please try again.");
        }
    }
}
