use anyhow::Result;
use axum::{
    Extension, Json, Router,
    extract::Path as axum_path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use dotenvy::dotenv;
use sea_orm::{prelude::*, *};
use sea_orm_migration::prelude::*;
use std::{fs, path::Path, sync::Arc, time::Duration};
use tower_http::services::ServeDir;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

use migration::{Migrator, MigratorTrait};

mod db;
use db::{
    entities::{ImageModel, TagModel},
    repositories::{IImageRepository, ITagRepository, ImageRepository, TagRepository},
};

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
    let images_repo: Arc<dyn IImageRepository + Send + Sync> =
        Arc::new(ImageRepository::new(db.clone()));
    let tags_repo: Arc<dyn ITagRepository + Send + Sync> = Arc::new(TagRepository::new(db.clone()));
    tracing::info!("Database configured successfully.");

    tracing::info!("Configuring application");
    let app = setup_router()
        .layer(Extension(db))
        .layer(Extension(images_repo))
        .layer(Extension(tags_repo));
    tracing::info!("Application configured successfully.");

    tracing::info!("Starting server");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on http://localhost:3000");
    axum::serve(listener, app).await?;
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

fn setup_router() -> Router {
    let static_path = std::env::current_dir().unwrap().join("wwwroot");
    tracing::info!("Configuring router");
    Router::new()
        .route("/", get(image_list))
        .route("/count", get(image_count))
        .route("/", post(image_add))
        .route("/{id}", get(image_get))
        .route("/{id}", put(image_update))
        .route("/{id}", delete(image_delete))
        .route("/{id}/tags/", get(image_tag_list))
        .route("/{id}/tags/", post(image_tag_add))
        .route("/{id}/tags/{tag_id}", delete(image_tag_remove))
        .route("/tags/", get(tag_list))
        .route("/tags/count", get(tag_count))
        .route("/tags/", post(tag_add))
        .route("/tags/{id}", get(tag_get))
        .route("/tags/{id}", put(tag_update))
        .route("/tags/{id}", delete(tag_delete))
        .route("/tags/{id}/images/", get(tag_image_list))
        .route("/tags/{id}/images/", post(tag_image_add))
        .route("/tags/{id}/images/{tag_id}", delete(tag_image_remove))
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
}

// Handlers
async fn image_list(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
) -> Result<Json<Vec<ImageModel>>, (StatusCode, String)> {
    match repo.list().await {
        Ok(images) => Ok(Json(images)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_count(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
) -> Result<Json<u64>, (StatusCode, String)> {
    match repo.count().await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_add(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    Json(image): Json<ImageModel>,
) -> Result<Json<ImageModel>, (StatusCode, String)> {
    match repo.create(image).await {
        Ok(created) => Ok(Json(created)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_get(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<ImageModel>, (StatusCode, String)> {
    match repo.get(id).await {
        Ok(Some(image)) => Ok(Json(image)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Image not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_update(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
    Json(image): Json<ImageModel>,
) -> Result<Json<ImageModel>, (StatusCode, String)> {
    match repo.update(id, image).await {
        Ok(updated) => Ok(Json(updated)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_delete(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.delete(id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_tag_list(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<Vec<TagModel>>, (StatusCode, String)> {
    match repo.list_tags(id).await {
        Ok(tags) => Ok(Json(tags)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_tag_add(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path((id, tag_id)): axum_path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.add_tag(id, tag_id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_tag_remove(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path((id, tag_id)): axum_path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.remove_tag(id, tag_id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_list(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
) -> Result<Json<Vec<TagModel>>, (StatusCode, String)> {
    match repo.list().await {
        Ok(tags) => Ok(Json(tags)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_count(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
) -> Result<Json<u64>, (StatusCode, String)> {
    match repo.count().await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_add(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    Json(tag): Json<TagModel>,
) -> Result<Json<TagModel>, (StatusCode, String)> {
    match repo.create(tag).await {
        Ok(created) => Ok(Json(created)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_get(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<TagModel>, (StatusCode, String)> {
    match repo.get(id).await {
        Ok(Some(tag)) => Ok(Json(tag)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Tag not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_update(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
    Json(tag): Json<TagModel>,
) -> Result<Json<TagModel>, (StatusCode, String)> {
    match repo.update(id, tag).await {
        Ok(updated) => Ok(Json(updated)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_delete(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.delete(id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_image_list(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<Vec<ImageModel>>, (StatusCode, String)> {
    match repo.list_images(id).await {
        Ok(images) => Ok(Json(images)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_image_add(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path((id, image_id)): axum_path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.add_image(id, image_id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_image_remove(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path((id, image_id)): axum_path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.remove_image(id, image_id).await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
