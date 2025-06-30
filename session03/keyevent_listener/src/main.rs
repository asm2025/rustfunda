use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{sync::mpsc, thread, time::Duration};

fn main() {
    let (tx, rx) = mpsc::channel();

    // Spawn key listener thread
    thread::spawn(move || {
        enable_raw_mode().unwrap();

        loop {
            if let Ok(Event::Key(key)) = event::read() {
                if !key.is_press() {
                    continue;
                }

                if tx.send(key).is_err() {
                    // Main thread dropped
                    break;
                }
            }
        }

        disable_raw_mode().unwrap();
    });

    println!("Press keys (ESC to quit):");

    // Main thread continues without blocking
    loop {
        match rx.try_recv() {
            Ok(key) => match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => {
                    if key.modifiers.is_empty() {
                        println!("Pressed: {}", c);
                    } else {
                        println!("Pressed: {} with {:?}", c, key.modifiers);
                    }
                }
                _ => println!("Pressed: {:?} with {:?}", key.code, key.modifiers),
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                // Listener is disconnected
                break;
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
