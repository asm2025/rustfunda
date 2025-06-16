use crossbeam::channel::{self, Receiver, Sender};
use fake::{Fake, Faker};
use std::{thread, time::Duration};
use util::auth::User;

fn main() {
    let threads = num_cpus::get();
    let n_users = threads * 4;
    let (tx, rx): (Sender<User>, Receiver<User>) = channel::unbounded();
    println!("Spawning {} consumers...", threads);
    thread::scope(|scope| {
        // Consumer threads
        for i in 0..threads {
            let n = i + 1;
            let rx2 = rx.clone();
            scope.spawn(move || {
                println!("CNS {}>>> Starting up.", n);

                while let Ok(user) = rx2.recv() {
                    println!("CNS {}>>> Processing user: {}", n, user);
                    thread::sleep(Duration::from_millis(300));
                }

                println!("CNS {}>>> Shutting down.", n);
            });
        }

        // Producer thread
        scope.spawn(move || {
            println!("\nProducer starting to generate {} users...", n_users);

            for i in 0..n_users {
                let n = i + 1;
                let user: User = Faker.fake();
                println!("PRD >>> Enqueueing user {}.", n);
                tx.send(user).expect(&format!("Failed to send user {}.", n));
                thread::sleep(Duration::from_millis(50));
            }

            println!("Producer finished.");
            drop(tx);
        });
    });
    println!("All threads are completed.");
}
