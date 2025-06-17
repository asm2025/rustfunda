use rayon::ThreadPoolBuilder;

fn main() {
    let threads = num_cpus::get();
    let pool = ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap();
    pool.spawn(|| println!("Hello from thread pool."));
    pool.scope(|scope| {
        for n in 0..20 {
            scope.spawn(move |_| {
                println!("Hello from scoped thread {n}");
            });
        }

        scope.spawn_broadcast(|_, context| {
            println!("Hello from broadcast thread {}", context.index());
        });
    });
    println!("Hello from main thread");
}
