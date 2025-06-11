use std::{thread, time::Instant};

fn hello_thread(n: usize) -> (usize, usize) {
    let current_thread = thread::current();
    let id = format!("{:?}", current_thread.id());
    let thread_name = current_thread.name().unwrap_or(&id);
    println!("Hello from {thread_name} thread!");

    let mut sum = n + 1;

    for _ in 0..10 {
        sum *= 2;
    }

    (n, sum)
}

fn main() {
    println!("Hello from main thread!");

    let mut handles = vec![];

    for i in 0..10 {
        let start = Instant::now();
        let handle = thread::Builder::new()
            .name(format!("My Thread {i:02}"))
            .spawn(move || hello_thread(i))
            .unwrap();
        let creation_time = start.elapsed();
        println!("Thread {i}: {:?} nanosecond", creation_time.as_nanos());
        handles.push(handle);
    }

    for handle in handles {
        let (n, sum) = handle.join().unwrap();
        println!("Thread {n} sum: {sum}");
    }

    println!("Thread has finished execution.");
}
