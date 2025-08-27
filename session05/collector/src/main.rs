mod collector;

use collector::Collector;
use shared_data::CollectorCommand;
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};

fn main() {
    const TRIES: u32 = 100;
    const ERRORS: u32 = 3;

    let (tx, rx) = mpsc::sync_channel::<shared_data::CollectorCommand>(10);
    let collector_id = shared_data::new_collector_id();
    let mut collector = Collector::new(collector_id);
    let sender = Arc::new(tx);
    let handle = collector.start(sender, Duration::from_secs(1)).unwrap();

    let mut messages = TRIES;
    let mut errors = ERRORS;

    'main_loop: loop {
        match rx.recv() {
            Ok(command) => match collector.publish(&command) {
                Ok(_) => {
                    messages -= 1;
                    errors = ERRORS;

                    if messages == 0 {
                        let command = CollectorCommand::Exit { collector_id };
                        let _ = collector.publish(&command);
                        break 'main_loop;
                    }
                }
                Err(ex) => {
                    errors -= 1;

                    if errors == 0 {
                        println!("Maximum errors sending to server exceeded. {}", ex);
                        break;
                    } else {
                        println!("{}", ex);
                    }
                }
            },
            Err(_) => {
                break 'main_loop;
            }
        }
    }

    collector.stop();
    let _ = handle.join();
}
