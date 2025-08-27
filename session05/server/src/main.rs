mod receiver;

use anyhow::Result;
use axum::{
    Extension, Json, Router,
    extract::Path as axum_path,
    http::HeaderValue,
    routing::{delete, get},
};
use dotenvy::dotenv;
use receiver::Receiver;
use shared_data::{Collector, CollectorCommand, DataPoint, Metrics};
use sqlx::{
    Pool,
    migrate::MigrateDatabase,
    sqlite::{Sqlite, SqlitePool, SqliteQueryResult},
};
use std::{
    fs,
    path::Path,
    sync::{Arc, mpsc},
};
use tokio::task::JoinHandle;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
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
    tracing::info!("Database configured successfully.");

    let metrics_handle = watch_metrics(&db).await;

    tracing::info!("Configuring application");
    let app = setup_router().layer(Extension(db.clone()));
    tracing::info!("Application configured successfully.");

    let server_handle = run_server(app).await;

    let (metrics_res, server_res) = tokio::join!(metrics_handle, server_handle);

    if let Err(err) = metrics_res {
        tracing::error!("Metrics task failed: {:?}", err);
    }

    if let Err(err) = server_res {
        tracing::error!("Server task failed: {:?}", err);
    }

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

fn setup_router() -> Router {
    let curdir = std::env::current_dir().unwrap();
    let static_path = curdir.join("wwwroot");
    let origins = std::env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| "http://localhost".to_string())
        .split(',')
        .map(|s| s.trim().parse::<HeaderValue>().unwrap())
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any);

    tracing::info!("Configuring router");
    Router::new()
        .route("/api/collectors", get(web::show_collectors))
        .route(
            "/api/collectors/{uuid}",
            get(web::show_metrics_by_collector),
        )
        .route("/api/metrics", get(web::show_metrics))
        .route("/api/metrics", delete(web::clear_metrics))
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
        .layer(cors)
}

// collector loop
async fn watch_metrics(db: &Pool<Sqlite>) -> JoinHandle<()> {
    let (tx, rx) = mpsc::sync_channel::<(u128, CollectorCommand)>(10);
    let mut receiver = Receiver::new();
    let sender = Arc::new(tx);
    let handle = receiver.start(sender).unwrap();
    let db = db.clone();
    tokio::spawn(async move {
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
                        let result =
                            data::add_metrics(&db, &collector_id, timestamp, &metrics).await;

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
    })
}

// server loop
async fn run_server(app: Router) -> JoinHandle<()> {
    tracing::info!("Starting server");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on http://localhost:3000");
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    })
}

mod data {
    use super::*;

    pub async fn get_collectors(db: &Pool<Sqlite>) -> Result<Vec<Collector>> {
        const SQL: &str = "SELECT collector_id, 
    MAX(received) AS last_seen 
    FROM timeseries ts
	GROUP BY collector_id
	ORDER BY last_seen";
        let mut collectors = sqlx::query_as::<_, Collector>(SQL)
            .fetch_all(db)
            .await
            .unwrap();

        for collector in &mut collectors {
            let last_seen: u128 = collector.last_seen.parse().unwrap();
            collector.last_seen = datetime::format_seconds_long(last_seen);
        }

        Ok(collectors)
    }

    pub async fn get_metrics(db: &Pool<Sqlite>) -> Result<Vec<DataPoint>> {
        let mut data_points = sqlx::query_as::<_, DataPoint>("SELECT * FROM TIMESERIES")
            .fetch_all(db)
            .await
            .unwrap();

        for data_point in &mut data_points {
            let received: u128 = data_point.received.parse().unwrap();
            data_point.received = datetime::format_seconds_long(received);
        }

        Ok(data_points)
    }

    pub async fn get_metrics_by_collector(db: &Pool<Sqlite>, uuid: &str) -> Result<Vec<DataPoint>> {
        let mut data_points = sqlx::query_as::<_, DataPoint>(
            "SELECT * FROM timeseries WHERE collector_id = ? ORDER BY received",
        )
        .bind(uuid)
        .fetch_all(db)
        .await
        .unwrap();

        for data_point in &mut data_points {
            let received: u128 = data_point.received.parse().unwrap();
            data_point.received = datetime::format_seconds_long(received);
        }

        Ok(data_points)
    }

    pub async fn add_metrics(
        db: &Pool<Sqlite>,
        collector_id: &str,
        timestamp: u128,
        metrics: &Metrics,
    ) -> Result<SqliteQueryResult> {
        sqlx::query(
            "INSERT INTO TIMESERIES (
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
        .bind(timestamp as i64)
        .bind(metrics.total_memory as i64)
        .bind(metrics.used_memory as i64)
        .bind(metrics.cpus as i32)
        .bind(metrics.cpu_usage)
        .bind(metrics.avg_cpu_usage)
        .execute(db)
        .await
        .map_err(|ex| ex.into())
    }

    pub async fn clear_metrics(db: &Pool<Sqlite>) -> Result<SqliteQueryResult> {
        sqlx::query("DELETE FROM TIMESERIES")
            .execute(db)
            .await
            .map_err(|ex| ex.into())
    }
}

mod web {
    use super::*;

    pub async fn show_collectors(Extension(db): Extension<SqlitePool>) -> Json<Vec<Collector>> {
        let rows = data::get_collectors(&db).await.unwrap();
        Json(rows)
    }

    pub async fn show_metrics(Extension(db): Extension<SqlitePool>) -> Json<Vec<DataPoint>> {
        let rows = data::get_metrics(&db).await.unwrap();
        Json(rows)
    }

    pub async fn show_metrics_by_collector(
        Extension(db): Extension<SqlitePool>,
        uuid: axum_path<String>,
    ) -> Json<Vec<DataPoint>> {
        let rows = data::get_metrics_by_collector(&db, &uuid).await.unwrap();
        Json(rows)
    }

    pub async fn clear_metrics(Extension(db): Extension<SqlitePool>) {
        data::clear_metrics(&db).await.unwrap();
    }
}
