use std::{sync::mpsc, time::Duration};
use tokio::{sync::mpsc as tkmpsc, time::sleep as tksleep};

enum Command {
    Print(String),
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<Command>();
    let (tx_reply, mut rx_reply) = tkmpsc::channel::<String>(10);
    let handle = tokio::runtime::Handle::current();

    std::thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            match command {
                Command::Print(s) => {
                    let tx_reply = tx_reply.clone();
                    handle.spawn(async move {
                        tx_reply.send(s).await.unwrap();
                    });
                    //println!("{s}");
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(reply) = rx_reply.recv().await {
            println!("{reply}");
        }
    });

    let mut counter = 0u32;

    loop {
        tksleep(Duration::from_secs(1)).await;
        tx.send(Command::Print(format!("Count: {counter}")))
            .unwrap();
        counter += 1;
    }
}
