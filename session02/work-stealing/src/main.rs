use crossbeam::deque::{Injector, Stealer, Worker};
use fake::{Fake, Faker};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
    usize,
};
use util::auth::User;

fn producer(injector: Arc<Injector<User>>, shutdown: Arc<AtomicBool>, n_users: usize) {
    println!("\nProducer starting to generate {} users...", n_users);

    for i in 0..n_users {
        let n = i + 1;
        let user: User = Faker.fake();
        println!("PRD >>> Enqueueing user {}.", n);
        injector.push(user);
        thread::sleep(Duration::from_millis(50));
    }

    println!("Producer finished.");
    shutdown.store(true, Ordering::SeqCst);
}

fn consumer(
    n: usize,
    local: &Worker<User>,
    stealers: &[Stealer<User>],
    injector: &Injector<User>,
    shutdown: &AtomicBool,
) {
    println!("CNS {}>>> Starting up.", n);

    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        if let Some(user) = find_task(local, stealers, injector) {
            println!("CNS {}>>> Processing user: {}", n, user);
            thread::sleep(Duration::from_millis(300));
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }

    println!("CNS {}>>> Shutting down.", n);
}

fn find_task<'a>(
    local: &'a Worker<User>,
    stealers: &'a [Stealer<User>],
    injector: &'a Injector<User>,
) -> Option<User> {
    local.pop().or_else(|| {
        stealers
            .iter()
            .filter(|x| !x.is_empty())
            .map(|x| x.steal())
            .find(|x| x.is_success())
            .and_then(|x| x.success())
            .or_else(|| injector.steal().success())
    })
}

fn main() {
    let threads = num_cpus::get();
    let n_users = threads * 4;
    let injector: Arc<Injector<User>> = Arc::new(Injector::new());
    let workers: Vec<Worker<User>> = (0..n_users).map(|_| Worker::new_fifo()).collect();
    let stealers: Vec<Stealer<User>> = workers.iter().map(|w| w.stealer()).collect();
    let shutdown = Arc::new(AtomicBool::new(false));
    println!("Spawning {} consumers...", threads);
    thread::scope(|scope| {
        // Producer thread
        let injector2 = injector.clone();
        let shutdown2 = shutdown.clone();
        scope.spawn(move || {
            producer(injector2, shutdown2, n_users);
        });

        // Consumer threads
        for (i, worker) in workers.into_iter().enumerate() {
            let n = i + 1;
            let stealers = &stealers;
            let injector = &injector;
            let shutdown = &shutdown;
            scope.spawn(move || {
                consumer(n, &worker, stealers, injector, shutdown);
            });
        }
    });
    println!("All threads are completed.");
}
