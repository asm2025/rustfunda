mod collector;

use collector::Collector;
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};
use util;

fn main() {
    let (tx, rx) = mpsc::channel::<shared_data::CollectorCommand>();
    let collector_id = shared_data::new_collector_id();
    let mut collector = Collector::new(collector_id);
    let mut messages = 10u32;
    let sender = Arc::new(tx);
    let handle = collector.start(sender, Duration::from_secs(1)).unwrap();

    while let Ok(command) = rx.recv() {
        collector.send(command);
        messages -= 1;

        if messages == 0 {
            break;
        }
    }

    collector.stop();
    let _ = handle.join();
}

// fn on_metrics(t: u64, m: shared_data::Metrics) {
//     let time = util::datetime::format_seconds_with_precision(t);
//     println!(
//         "{} mem: {}/{} KB, CPUs: {}, CPU usage: {:.2}%, CPU usage (avg): {:.2}%",
//         time, m.used_memory, m.total_memory, m.cpus, m.cpu_usage, m.avg_cpu_usage
//     );
// }
