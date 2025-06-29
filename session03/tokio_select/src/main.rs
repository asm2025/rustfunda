use tokio::{
    select,
    sync::{broadcast, mpsc},
    time::{Duration, sleep},
};

async fn do_work(duration: u64) {
    sleep(Duration::from_millis(duration)).await;
}

async fn receiver(mut rx: mpsc::Receiver<u32>, mut bcrx: broadcast::Receiver<u32>) {
    loop {
        select! {
            Some(n) = rx.recv() => println!("Received message {n} on the mpsc channel."),
            Ok(n) = bcrx.recv() => println!("Received message {n} on the broadcast channel."),
        }
    }
}

#[tokio::main]
async fn main() {
    select! {
        _ = do_work(100) => println!("do_work + 100 finished first"),
        _ = do_work(200) => println!("do_work + 200 finished first"),
        _ = do_work(400) => println!("do_work + 400 finished first"),
    }

    let (tx, rx) = mpsc::channel::<u32>(1);
    let (bctx, bcrx) = broadcast::channel::<u32>(1);
    tokio::spawn(receiver(rx, bcrx));

    for n in 0..10 {
        if n % 2 == 0 {
            tx.send(n).await.unwrap();
        } else {
            bctx.send(n).unwrap();
        }

        sleep(Duration::from_secs(1)).await;
    }
}
