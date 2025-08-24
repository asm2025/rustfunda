mod receiver;

use receiver::Receiver;
use shared_data::CollectorCommand;
use std::sync::{Arc, mpsc};
use util::{Result, datetime};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::sync_channel::<(u128, shared_data::CollectorCommand)>(10);
    let mut receiver = Receiver::new();
    let sender = Arc::new(tx);
    let handle = receiver.start(sender).unwrap();

    'main_loop: loop {
        match rx.recv() {
            Ok((timestamp, command)) => match command {
                CollectorCommand::SubmitData {
                    collector_id,
                    metrics,
                } => println!(
                    "{} {} mem: {}/{} KB, CPUs: {}, CPU usage: {:.2}%, CPU usage (avg): {:.2}%",
                    datetime::format_seconds_long(timestamp),
                    collector_id,
                    metrics.used_memory,
                    metrics.total_memory,
                    metrics.cpus,
                    metrics.cpu_usage,
                    metrics.avg_cpu_usage
                ),
                CollectorCommand::Exit { collector_id } => {
                    println!("Closing connection to {collector_id}");
                    break 'main_loop;
                }
            },
            Err(ex) => {
                println!("{}", ex);
                break 'main_loop;
            }
        }
    }

    receiver.stop();
    let _ = handle.join();

    Ok(())
}
