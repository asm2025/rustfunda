use std::{sync::mpsc, thread};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Command {
    Run(Job),
    Exit,
}

fn hi_there() {
    println!("Hello from the worker thread!");
}

fn main() {
    let (tx, rx) = mpsc::channel::<Command>();
    let handle = thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            match command {
                Command::Run(job) => {
                    job();
                }
                Command::Exit => {
                    println!("Exiting...");
                    break;
                }
            }
        }
    });
    let job = || println!("Hello from my closure!");
    let job2 = || {
        for i in 1..=5 {
            println!("Job 2: {}", i);
        }
    };
    tx.send(Command::Run(Box::new(hi_there))).unwrap();
    tx.send(Command::Run(Box::new(job))).unwrap();
    tx.send(Command::Run(Box::new(job2))).unwrap();
    tx.send(Command::Run(Box::new(|| println!("I'm in the box!"))))
        .unwrap();
    tx.send(Command::Exit).unwrap();
    handle.join().unwrap();
}
