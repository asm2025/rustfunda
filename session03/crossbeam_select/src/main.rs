use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    select,
    sync::{broadcast, mpsc},
    time::{Duration, sleep},
};
use util::{Result, io::KeyListener};

async fn do_work(duration: u64) {
    sleep(Duration::from_millis(duration)).await;
}

async fn receiver(
    mut rx: mpsc::Receiver<u32>,
    mut bcrx: broadcast::Receiver<u32>,
    cancelled: Arc<AtomicBool>,
) {
    loop {
        if cancelled.load(Ordering::Relaxed) {
            println!("Receiver found a cancellation flag. Shutting down.");
            break;
        }

        select! {
            Some(n) = rx.recv() => println!("Received message {n} on the mpsc channel."),
            Ok(n) = bcrx.recv() => println!("Received message {n} on the broadcast channel."),
            _ = sleep(Duration::from_millis(100)) => {},
            else => break,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    select! {
        _ = do_work(100) => println!("do_work + 100 finished first"),
        _ = do_work(200) => println!("do_work + 200 finished first"),
        _ = do_work(400) => println!("do_work + 400 finished first"),
    }

    let (tx, rx) = mpsc::channel::<u32>(1);
    let (bctx, bcrx) = broadcast::channel::<u32>(1);
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled2 = cancelled.clone();
    let mut key_listener = KeyListener::new().unwrap();
    let receiver_handle = tokio::spawn(receiver(rx, bcrx, cancelled2));
    println!("\nPress any key to cancel the loop...\n");

    'main_loop: for n in 0..100 {
        select! {
            // This branch listens for the signal from the keyboard thread.
            biased;
            Some(_) = key_listener.recv() => {
                println!("Key press received in main. Breaking loop.");
                cancelled.store(true, Ordering::Relaxed);
                break 'main_loop;
            }
            // This branch is the main work of sending messages and sleeping.
            _ = async {
                if n % 2 == 0 {
                    let _ = tx.send(n).await;
                } else {
                    let _ = bctx.send(n);
                }
                sleep(Duration::from_secs(1)).await;
            } => { /* Work for this iteration completed */ }
        }
    }

    if cancelled.load(Ordering::Relaxed) {
        println!("Loop was cancelled by user.");
    } else {
        println!("Loop finished naturally.");
    }

    println!("Broadcasting final shutdown signal...");
    cancelled.store(true, Ordering::Relaxed);
    let _ = receiver_handle.await;
    println!("All tasks finished gracefully.");
    Ok(())
}
