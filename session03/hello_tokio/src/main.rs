use tokio::{
    runtime,
    time::{Duration, Instant, sleep},
};

async fn hello() {
    println!("Hello");
}

fn main() {
    let threads = ((num_cpus::get() as f64 * 0.6).max(1.0)).ceil() as usize;
    println!("Using {threads} threads");

    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(threads)
        .build()
        .unwrap();
    let now = Instant::now();
    rt.block_on(async {
        hello().await;
        println!("Hello from a manually configured async runtime!");

        let mut handles = vec![];

        for i in 0..threads {
            let handle = tokio::spawn(async move {
                println!("Task {} is running.", i);
                sleep(Duration::from_millis(10)).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    });
    println!(
        "All tasks have finished in {} seconds.",
        now.elapsed().as_secs_f64()
    );
}
