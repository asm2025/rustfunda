use crossterm::event::KeyCode;
use std::{thread, time::Duration};
use util::{Result, io::KeyListener, sync::mpsc::error::TryRecvError};

fn main() -> Result<()> {
    let mut key_listener = KeyListener::new()?;
    println!("Press keys (ESC to quit):");

    // Main thread continues without blocking
    loop {
        match key_listener.try_recv() {
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
            Err(TryRecvError::Disconnected) => {
                // Listener is disconnected
                break;
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    Ok(())
}
