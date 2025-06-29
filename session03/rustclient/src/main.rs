use anyhow::Result;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{Duration, timeout},
};
use util::io;

async fn get_my_ip() -> Result<String> {
    const URL: &'static str = "https://httpbin.org/ip";

    #[derive(Debug, Deserialize)]
    struct IPAddress {
        origin: String,
    }

    let resp = reqwest::get(URL).await?.json::<IPAddress>().await?;
    Ok(resp.origin)
}

async fn get_some_text() -> Result<String> {
    const URL: &'static str = "https://httpbin.org/html";

    let resp = reqwest::get(URL).await?.text().await?;
    Ok(resp)
}

async fn get_weather() -> Result<JsonValue> {
    const URL: &'static str = "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&hourly=temperature_2m&current=temperature_2m,relative_humidity_2m,rain,showers,snowfall&timezone=Africa%2FCairo";

    let resp = reqwest::get(URL).await?;
    let json = resp.json::<JsonValue>().await?;
    Ok(json)
}

async fn connect_to_tcp() -> Result<()> {
    const HOST: &'static str = "127.0.0.1:8123";
    const BUFFER_SIZE: usize = 1024;

    let mut stream = TcpStream::connect(HOST).await?;
    println!();
    println!("Connected to {}", HOST);

    let mut buffer = vec![0u8; BUFFER_SIZE];

    if let Ok(Ok(n)) = timeout(Duration::from_millis(500), stream.read(&mut buffer)).await {
        let response = String::from_utf8_lossy(&buffer[..n]);
        println!("{}", response.trim_end());
    };

    loop {
        let input = match io::get_str(Some("> ")) {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };
        stream.write_all(input.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        let n = match timeout(Duration::from_secs(1), stream.read(&mut buffer)).await {
            Ok(Ok(n)) => n,
            Ok(Err(_)) => continue,
            Err(_) => continue,
        };

        if n == 0 {
            println!("Server closed connection.");
            break;
        }

        let response = String::from_utf8_lossy(&buffer[..n]);
        println!("{}", response.trim_end());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let ip = get_my_ip().await?;
    println!("My IP address: {ip}");

    let example_text = get_some_text().await?;
    println!("{example_text}");

    println!("My weathr forcast:");
    let weather = get_weather().await?;
    println!("{weather:#?}");

    println!("Trying to connect to TCP server...");
    connect_to_tcp().await?;

    Ok(())
}
