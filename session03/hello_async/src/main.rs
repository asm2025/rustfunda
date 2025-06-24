use futures::{
    self,
    executor::block_on,
    future::{FutureExt, join_all, select_all},
};
use futures_timer::Delay;
use std::time::Duration;
use tokio::time::sleep;

async fn say_hello() {
    println!("Hello");
    futures::join!(second_function(), say_goodbye());

    let futures = vec![double(1), double(2), double(3)];
    let results = join_all(futures).await;
    println!("{results:?}");
}

async fn second_function() {
    println!("Hello again");
}

async fn say_goodbye() {
    println!("Goodbye");
}

async fn double(n: u32) -> u32 {
    n * 2
}

async fn do_work_fast() -> &'static str {
    sleep(Duration::from_millis(100)).await;
    "Fast work is done!"
}

async fn do_work_slow() -> &'static str {
    sleep(Duration::from_millis(200)).await;
    "Slow work is done!"
}

async fn do_work_fast_delay() -> &'static str {
    Delay::new(Duration::from_millis(100)).await;
    "Delayed fast work is done!"
}

async fn do_work_slow_delay() -> &'static str {
    Delay::new(Duration::from_millis(200)).await;
    "Delayed slow work is done!"
}

async fn sleep_and_return(duration_ms: u64, val: u32) -> u32 {
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    val
}

#[tokio::main]
async fn main() {
    block_on(say_hello());
    tokio::select! {
        result = do_work_fast() => {
            println!("{}", result);
        }
        result = do_work_slow() => {
            println!("{}", result);
        }
    }
    futures::select! {
        result = do_work_fast_delay().fuse() => {
            println!("{}", result);
        }
        result = do_work_slow_delay().fuse() => {
            println!("{}", result);
        }
    }
    let futures = vec![
        sleep_and_return(300, 1).boxed(),
        sleep_and_return(100, 2).boxed(), // This one is fastest
        sleep_and_return(200, 3).boxed(),
    ];
    let (result, index, _remaining_futures) = select_all(futures).await;
    println!(
        "Future at index {} finished first with the result {}.",
        index, result
    );
}
