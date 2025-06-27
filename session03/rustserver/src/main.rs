use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    spawn,
};

#[tokio::main]
async fn main() -> Result<()> {
    const HOST: &'static str = "127.0.0.1:8123";
    const BUFFER_SIZE: usize = 1024;

    let listener = TcpListener::bind(HOST).await?;
    println!();
    println!("Listening on {}", HOST);
    println!("You can use PuTTY or any TCP client to send mesages to this server.");
    println!(
        "If you see strange squares when first connected, try to make a RAW connection instead of Telnet."
    );
    println!();

    loop {
        let (mut socket, address) = listener.accept().await?;
        spawn(async move {
            println!("Connection from {address:?}");
            let welcome = b"Welcome to the Rust TCP server!\r\nType something and it will be echoed back.\r\nSend 'QUIT' to exit.\r\n";

            if let Err(e) = socket.write_all(welcome).await {
                eprintln!("Failed to write welcome message: {e}");
                return;
            }

            let mut buffer = vec![0; BUFFER_SIZE];

            loop {
                let n = socket
                    .read(&mut buffer)
                    .await
                    .expect("Failed to read data from the socket!");

                if n == 0 {
                    println!("Closing connection from {address:?}");
                    return;
                }

                let message = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

                if message.is_empty() {
                    continue;
                }

                println!("{message}");

                if message.eq_ignore_ascii_case("QUIT") {
                    println!("Received QUIT, closing connection from {address:?}");
                    return;
                }
            }
        });
    }
}
