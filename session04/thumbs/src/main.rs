use anyhow::Result;
use axum::{Extension, Router};
use dotenvy::dotenv;
use sea_orm::{prelude::*, *};
use sea_orm_migration::prelude::*;
use serde_json::{Value as JsonValue, json};
use std::{
    fs,
    path::{MAIN_SEPARATOR, Path},
    time::Duration,
};
use tower_http::services::ServeDir;

mod entities;
use entities::prelude::*;
// > sea-orm-cli migrate init
use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt::init();
    tracing::info!("Starting the application...");

    // Setup the database connection
    let db_url = std::env::var("DATABASE_URL")?;
    let db = setup_database(&db_url).await?;
    tracing::info!("Database connection established.");

    let app = create_router().layer(Extension(db));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

// Setup the database connection
async fn setup_database(db_url: &str) -> Result<DatabaseConnection> {
    let db_path = db_url.strip_prefix("sqlite://").unwrap_or(&db_url);

    // Check if the parent directory exists
    if db_path.contains(MAIN_SEPARATOR) || db_path.contains('/') {
        if let Some(parent) = Path::new(db_path).parent() {
            if !parent.as_os_str().is_empty() {
                // Create the directory if it doesn't exist
                fs::create_dir_all(parent)?;
                tracing::info!("Created directory for database: {}", parent.display());
            }
        }
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
    Migrator::up(&db, None).await?;

    Ok(db)
}

// Setup the router
fn create_router() -> Router {
    let static_path = std::env::current_dir().unwrap().join("wwwroot");
    Router::new()
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
}
