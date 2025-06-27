use anyhow::{Ok, Result};
use serde::Deserialize;
use serde_json::Value as JsonValue;

async fn get_my_ip() -> Result<String> {
    const URL: &'static str = "https://httpbin.org/ip";

    #[derive(Debug, Deserialize)]
    struct IPAddress {
        origin: String,
    }

    let resp = reqwest::get(URL).await?.json::<IPAddress>().await?;
    Ok(resp.origin)
}

async fn get_weather() -> Result<JsonValue> {
    const URL: &'static str = "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&hourly=temperature_2m&current=temperature_2m,relative_humidity_2m,rain,showers,snowfall&timezone=Africa%2FCairo";

    let resp = reqwest::get(URL).await?;
    let json = resp.json::<JsonValue>().await?;
    Ok(json)
}

#[tokio::main]
async fn main() -> Result<()> {
    let ip = get_my_ip().await.unwrap();
    println!("My IP address: {ip}");

    println!("My weathr forcast:");
    let weather = get_weather().await?;
    println!("{weather:#?}");

    Ok(())
}
