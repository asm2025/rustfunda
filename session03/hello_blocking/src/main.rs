use std::time::Duration;
use tokio::task::spawn_blocking;

async fn hello_delay(task: u64, time: u64) {
    println!("Task {task} has started.");
    //std::thread::sleep(Duration::from_millis(time));
    //tokio::time::sleep(Duration::from_millis(time)).await;
    spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(time));
    })
    .await
    .unwrap();
    println!("Task {task} has finished.");
}

#[tokio::main]
async fn main() {
    tokio::join!(
        hello_delay(1, 500),
        hello_delay(2, 1000),
        hello_delay(3, 1500)
    );
}
