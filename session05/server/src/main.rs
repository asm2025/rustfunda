mod receiver;

use anyhow::Result;
use dotenvy::dotenv;
use receiver::Receiver;
use shared_data::CollectorCommand;
use sqlx::{
    Pool,
    migrate::MigrateDatabase,
    sqlite::{Sqlite, SqlitePool},
};
use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc},
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};
use util::datetime;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let app_name = env!("CARGO_PKG_NAME").to_string();
    setup_tracing(&app_name)?;

    tracing::info!("Starting {app_name}...");

    let result = run().await;

    if let Err(e) = result {
        tracing::error!("{app_name} error: {e}");
        std::process::exit(1);
    }

    tracing::info!("{app_name} shutdown.");
    Ok(())
}

async fn run() -> Result<()> {
    tracing::info!("Configuring database");
    let db_url = std::env::var("DATABASE_URL")?;
    let db = setup_database(&db_url).await?;

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
                } => {
                    let collector_id = Uuid::from_u128(collector_id);
                    let collector_id = collector_id.to_string();
                    println!(
                        "{} {} mem: {}/{} KB, CPUs: {}, CPU usage: {:.2}%, CPU usage (avg): {:.2}%",
                        datetime::format_seconds_long(timestamp),
                        collector_id,
                        metrics.used_memory,
                        metrics.total_memory,
                        metrics.cpus,
                        metrics.cpu_usage,
                        metrics.avg_cpu_usage
                    );
                    let result = sqlx::query(
                        "INSERT INTO timeseries (
							collector_id,
							received,
							total_memory,
							used_memory,
							cpus,
							cpu_usage,
							avg_cpu_usage
						)
						VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    )
                    .bind(collector_id)
                    .bind(timestamp.to_string())
                    .bind(metrics.total_memory as i64)
                    .bind(metrics.used_memory as i64)
                    .bind(metrics.cpus as i32)
                    .bind(metrics.cpu_usage)
                    .bind(metrics.avg_cpu_usage)
                    .execute(&db)
                    .await;

                    if result.is_err() {
                        println!("Error inserting metrics into the database. {result:?}")
                    }
                }
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

// Setup
fn setup_tracing(name: &str) -> Result<()> {
    // Create a directory for logs if it doesn't exist
    fs::create_dir_all("_logs")?;

    // Setup file appender for logging
    let log_filename = name.to_owned();
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "_logs", &log_filename);
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::TRACE
    } else {
        LevelFilter::INFO
    };
    let filter = EnvFilter::from_default_env()
        .add_directive("sqlx::query=off".parse()?)
        .add_directive("sqlx_core=off".parse()?)
        .add_directive(log_level.into());

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .compact()
                .with_file(true)
                .with_line_number(true)
                .with_thread_names(true)
                .with_target(false),
        )
        .with(
            fmt::layer().with_writer(file_appender).with_ansi(false), // No color codes in file
        )
        .init();

    Ok(())
}

async fn setup_database(db_url: &str) -> Result<Pool<Sqlite>> {
    let db_path = if let Some(pos) = db_url.find("://") {
        &db_url[pos + 3..]
    } else {
        db_url
    };

    let path = Path::new(db_path);

    if !path.exists() {
        // Check if the parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                // Create the directory if it doesn't exist
                fs::create_dir_all(parent)?;
                tracing::info!("Created directory for database: {}", parent.display());
            }
        }

        // Touch the file to ensure it can be created
        Sqlite::create_database(db_url).await?;
        tracing::info!("Created database file: {}", db_path);
    }

    // Create connection pool
    let pool = SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true),
    )
    .await?;
    tracing::info!("Connected to the database at {}", db_url);

    let path = Path::new("./migrations");

    if path.exists() {
        // Apply migrations
        tracing::info!("Applying migrations...");
        sqlx::migrate!("./migrations").run(&pool).await?;
        tracing::info!("Migrations applied successfully.");
    }

    Ok(pool)
}
