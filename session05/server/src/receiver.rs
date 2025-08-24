use shared_data::{CollectorCommand, DATA_COLLECTION_ADDRESS};
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
    },
    thread::{self, JoinHandle},
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    runtime::Builder,
    sync::Notify,
    task::{self, LocalSet},
};
use util::{Result, error::RmxError};

#[derive(Debug, Clone)]
pub struct Receiver {
    running: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl Receiver {
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(false));
        Self {
            running,
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn start(
        &mut self,
        sender: Arc<SyncSender<(u128, CollectorCommand)>>,
    ) -> Result<JoinHandle<()>> {
        if self
            .running
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(RmxError::InvalidOperation(
                "Receiver is already running.".to_string(),
            ));
        }

        let running = self.running.clone();
        let notify = self.notify.clone();
        let sender = sender.clone();
        let handle = thread::Builder::new()
            .name("receiver worker".to_string())
            .spawn(move || {
                let rt = Builder::new_current_thread().enable_all().build().unwrap();
                let local = LocalSet::new();
                local.block_on(&rt, async move {
                    task::spawn_local(async move {
                        let listener = TcpListener::bind(DATA_COLLECTION_ADDRESS).await.unwrap();
                        println!("Listening on {DATA_COLLECTION_ADDRESS}");

						loop {
							tokio::select! {
								res = listener.accept() => {
									match res {
										Ok((socket, address)) => {
											tokio::spawn(Self::new_connection(socket, address, sender.clone()));
										}
										Err(_) => {
											tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
										}
									}
								}
								_ = notify.notified() => {
									break;
								}
							}
						}

                        println!("Exiting listener loop");
                        running.store(false, Ordering::Release);
                    })
                    .await
                    .unwrap();
                });
            })
            .expect("failed to spawn receiver thread");
        Ok(handle)
    }

    pub fn stop(&mut self) {
        if !self.is_running() {
            return;
        }

        println!("Stopping the receiver.");
        self.notify.notify_waiters();
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    async fn new_connection(
        mut socket: TcpStream,
        address: SocketAddr,
        sender: Arc<SyncSender<(u128, CollectorCommand)>>,
    ) {
        println!("New connection from {address:?}.");

        let mut buffer = vec![0u8; 1024];

        loop {
            let n = match socket.read(&mut buffer).await {
                Ok(n) => n,
                Err(ex) => {
                    println!("{}", ex);
                    continue;
                }
            };

            if n == 0 {
                return;
            }

            println!("Recieved {n} bytes.");

            match shared_data::decode(&buffer[0..n]) {
                Ok((timestamp, command)) => {
                    let _ = sender.send((timestamp, command));
                }
                Err(ex) => println!("{}", ex),
            };
        }
    }
}
