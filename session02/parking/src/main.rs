use std::thread;
use util::{io::get_numeric, threading::Signal};

fn parkable(n: usize, signal: Signal) {
    loop {
        thread::park();
        println!("{n}>>> unparked briefly.");
        signal.set();
    }
}

fn main() {
    let mut threads = vec![];

    for i in 0..10 {
        let signal = Signal::new();
        let signal2 = signal.clone();
        threads.push((
            thread::spawn(move || {
                parkable(i + 1, signal2);
            }),
            signal,
        ));
    }

    loop {
        let input = get_numeric::<usize>(Some("Enter a number to unpark a thread (0 to exit): "))
            .unwrap_or(0);

        if input == 0 {
            break;
        }

        let index = input - 1;

        if index < threads.len() {
            let (handle, signal) = &threads[index];
            println!("Unparking thread {input}.");
            handle.thread().unpark();
            signal.wait();
        } else {
            println!("Invalid thread number: {input}. Please try again.");
        }
    }
}
