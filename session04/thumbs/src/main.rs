use anyhow::Result;
use axum::{Extension, Router};
use dotenvy::dotenv;
use sea_orm::{prelude::*, *};
use sea_orm_migration::prelude::*;
use std::{fs, net::SocketAddr, path::Path, time::Duration};
use tower_http::services::ServeDir;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

mod entities;
use migration::{Migrator, MigratorTrait};

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
    // Setup the database connection
    let db_url = std::env::var("DATABASE_URL")?;
    let db = setup_database(&db_url).await?;

    tracing::info!("Configuring application...");
    let app = create_router().layer(Extension(db));
    tracing::info!("Application configured successfully.");

    tracing::info!("Starting server...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

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

async fn setup_database(db_url: &str) -> Result<DatabaseConnection> {
    let db_path = if let Some(pos) = db_url.find("://") {
        &db_url[pos + 3..]
    } else {
        db_url
    };

    if !Path::new(db_path).exists() {
        // Check if the parent directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                // Create the directory if it doesn't exist
                fs::create_dir_all(parent)?;
                tracing::info!("Created directory for database: {}", parent.display());
            }
        }

        // Touch the file to ensure it can be created
        fs::File::create(db_path)?;
        tracing::info!("Created database file: {}", db_path);
    }

    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8));

    // Connect to the database
    let db = Database::connect(opt).await?;
    tracing::info!("Connected to the database at {}", db_url);

    // Apply migrations
    tracing::info!("Applying migrations...");
    Migrator::up(&db, None).await?;
    tracing::info!("Migrations applied successfully.");

    Ok(db)
}

// Setup the router
fn create_router() -> Router {
    let static_path = std::env::current_dir().unwrap().join("wwwroot");
    Router::new()
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
}
