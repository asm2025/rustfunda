use crossbeam::channel::{Receiver, bounded};
use std::thread;
use tokio::{
    select,
    sync::{broadcast, mpsc},
    time::{Duration, sleep},
};
use util::io::get_key;

async fn do_work(duration: u64) {
    sleep(Duration::from_millis(duration)).await;
}

async fn check_key_press(key_rx: &Receiver<()>) -> bool {
    match key_rx.try_recv() {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn receiver(
    mut rx: mpsc::Receiver<u32>,
    mut bcrx: broadcast::Receiver<u32>,
    mut cancelrx: broadcast::Receiver<()>,
) {
    loop {
        select! {
            _ = cancelrx.recv() => {
                println!("Receiver got cancellation signal. Shutting down.");
                break;
            }
            Some(n) = rx.recv() => println!("Received message {n} on the mpsc channel."),
            Ok(n) = bcrx.recv() => println!("Received message {n} on the broadcast channel."),
            else => break,
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
    println!("Key press is not working!!");

    let (tx, rx) = mpsc::channel::<u32>(1);
    let (bctx, bcrx) = broadcast::channel::<u32>(1);
    let (cancel_tx, _) = broadcast::channel::<()>(1);
    let (key_tx, key_rx) = bounded::<()>(1);

    thread::spawn(move || {
        if let Ok(_) = get_key(Some("\nPress any key to cancel the loop...\n")) {
            match key_tx.send(()) {
                Ok(_) => println!("Key press signal sent successfully"),
                Err(e) => println!("Failed to send key press signal: {}", e),
            }
        }
    });

    let cancel_rx = cancel_tx.subscribe();
    let receiver_handle = tokio::spawn(receiver(rx, bcrx, cancel_rx));
    let mut was_cancelled = false;

    'main_loop: for n in 0..100 {
        select! {
            biased;
            // Check for key press during the async work
            _ = async {
                // Check before sending
                if check_key_press(&key_rx).await {
                    return; // Exit this async block to trigger cancellation
                }

                if n % 2 == 0 {
                    let _ = tx.send(n).await;
                } else {
                    let _ = bctx.send(n);
                }

                // Check during sleep by sleeping in smaller chunks
                for _ in 0..10 {
                    if check_key_press(&key_rx).await {
                        return; // Exit this async block to trigger cancellation
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            } => {
                // Check one more time after the async block completes
                if check_key_press(&key_rx).await {
                    println!("Cancellation signal received in main. Breaking loop.");
                    was_cancelled = true;
                    break 'main_loop;
                }
            }
        }
    }

    if was_cancelled {
        println!("Loop was cancelled by user.");
    } else {
        println!("Loop finished naturally.");
    }

    println!("Broadcasting final shutdown signal...");
    let _ = cancel_tx.send(());
    let _ = receiver_handle.await;
    println!("All tasks finished gracefully.");
}
