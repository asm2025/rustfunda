use anyhow::Result;
use tracing::instrument;
use tracing_subscriber::{FmtSubscriber, fmt::format::FmtSpan};

#[instrument(name = "Hello")]
async fn hello_wolrd() {
    println!("Hello!");
}

#[tokio::main]
async fn main() -> Result<()> {
    // let subscriber = FmtSubscriber::new();
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_target(false)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT | FmtSpan::CLOSE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Starting up");
    tracing::warn!("Are you sure this is a good idea?");
    tracing::error!("Something went wrong.");

    hello_wolrd().await;

    Ok(())
}
