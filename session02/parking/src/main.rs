use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
};
use util::ninput;

fn parkable(n: usize, signal: Arc<(Mutex<bool>, Condvar)>) {
    loop {
        thread::park();
        println!(">>> Thread {n} was unparked briefly.");

        let (lock, cvar) = &*signal;
        let mut done = lock.lock().unwrap();
        *done = true;
        cvar.notify_one();
    }
}

fn main() {
    let mut threads = vec![];

    for i in 0..10 {
        let signal = Arc::new((Mutex::new(false), Condvar::new()));
        let signal2 = signal.clone();
        threads.push((
            thread::spawn(move || {
                parkable(i + 1, signal2);
            }),
            signal,
        ));
    }

    loop {
        let input =
            ninput::<usize>(Some("Enter a number to unpark a thread (0 to exit): ")).unwrap_or(0);

        if input == 0 {
            break;
        }

        let index = input - 1;

        if index < threads.len() {
            let (handle, signal) = &threads[index];
            let (lock, cvar) = &**signal;

            {
                let mut done = lock.lock().unwrap();
                *done = false;
            }

            println!("Unparking thread {input}.");
            handle.thread().unpark();

            let mut done = lock.lock().unwrap();

            while !*done {
                done = cvar.wait(done).unwrap();
            }
        } else {
            println!("Invalid thread number: {input}. Please try again.");
        }
    }
}
