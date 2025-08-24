use shared_data::{CollectorCommand, Metrics};
use std::{
    io::Write,
    net::TcpStream,
    panic,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use sysinfo::System;
use util::{Result, error::RmxError};

#[derive(Debug, Clone)]
pub struct Collector {
    pub collector_id: u128,
    running: Arc<AtomicBool>,
    stop_requested: Arc<AtomicBool>,
}

impl Collector {
    pub fn new(collector_id: u128) -> Self {
        let running = Arc::new(AtomicBool::new(false));
        let stop_requested = Arc::new(AtomicBool::new(false));
        Self {
            collector_id,
            running,
            stop_requested,
        }
    }

    pub fn start(
        &mut self,
        sender: Arc<SyncSender<CollectorCommand>>,
        period: Duration,
    ) -> Result<JoinHandle<()>> {
        if self
            .running
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(RmxError::InvalidOperation(
                "Collector is already running.".to_string(),
            ));
        }

        // Reset cancellation
        self.stop_requested.store(false, Ordering::Release);

        let collector_id = self.collector_id;
        let stop_requested = self.stop_requested.clone();
        let running = self.running.clone();
        let sender = sender.clone();
        let handle = thread::Builder::new()
            .name("collector worker".to_string())
            .spawn(move || {
                // Create sysinfo System inside the thread and refresh as needed.
                let mut sys = System::new_all();
                sys.refresh_all();

                let mut next_tick = Instant::now() + period;

                while !stop_requested.load(Ordering::Relaxed) {
                    let now = Instant::now();

                    if now < next_tick {
                        thread::sleep(next_tick - now);
                    }

                    next_tick += period;

                    let res = panic::catch_unwind(panic::AssertUnwindSafe({
                        let sender = sender.clone();
                        let sys_ref = &mut sys;
                        move || {
                            sys_ref.refresh_cpu_all();
                            sys_ref.refresh_memory();

                            let total_memory = sys_ref.total_memory();
                            let used_memory = sys_ref.used_memory();

                            let processors = sys_ref.cpus();
                            let num_cpus = processors.len();

                            let cpu_usage = sys_ref.global_cpu_usage();
                            let avg_cpu_usage = if num_cpus > 0 {
                                let sum: f32 = processors.iter().map(|p| p.cpu_usage()).sum();
                                sum / num_cpus as f32
                            } else {
                                cpu_usage
                            };

                            let metrics = Metrics {
                                total_memory,
                                used_memory,
                                cpus: num_cpus,
                                cpu_usage,
                                avg_cpu_usage,
                            };
                            let command = CollectorCommand::SubmitData {
                                collector_id,
                                metrics,
                            };
                            sender.send(command).unwrap();
                        }
                    }));

                    running.store(false, Ordering::Release);

                    if let Err(err) = res {
                        eprintln!("collector worker caught panic: {:?}", err);
                    }
                }
            })
            .expect("failed to spawn collector thread");
        Ok(handle)
    }

    pub fn stop(&mut self) {
        if self
            .stop_requested
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        println!("Stopping the collector.");
    }

    pub fn publish(&self, command: CollectorCommand) -> Result<()> {
        let bytes = shared_data::encode(command);
        println!("Sending {} bytes", bytes.len());

        let mut stream = TcpStream::connect(shared_data::DATA_COLLECTION_ADDRESS).map_err(|e| {
            RmxError::Network(format!(
                "Failed to connect to {}. {}",
                shared_data::DATA_COLLECTION_ADDRESS,
                e
            ))
        })?;
        stream.write_all(&bytes).map_err(|e| {
            RmxError::Network(format!(
                "Failed to send data to {}. {}",
                shared_data::DATA_COLLECTION_ADDRESS,
                e
            ))
        })?;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        self.stop();
    }
}
