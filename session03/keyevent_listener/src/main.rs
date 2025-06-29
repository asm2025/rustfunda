use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use util::io::clear_keys;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Spawn key listener thread
    thread::spawn(move || {
        println!("Press any key to stop the loop...");
        clear_keys();
        enable_raw_mode().unwrap();

        loop {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Ok(Event::Key(_)) = event::read() {
                    println!("\nKey pressed! Stopping loop...\n");
                    running_clone.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }

        disable_raw_mode().unwrap();
    });

    let mut counter = 0;

    while running.load(Ordering::Relaxed) {
        println!("Loop iteration: {}", counter);
        counter += 1;
        thread::sleep(Duration::from_secs(1));
    }

    println!("Loop finished!");
}
