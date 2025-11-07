use ::image::ImageReader;
use anyhow::Result;
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Multipart, Path as axum_path},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use dotenvy::dotenv;
use mime_guess::get_mime_extensions_str;
use sea_orm::{prelude::*, *};
use sea_orm_migration::prelude::*;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio_util::io::ReaderStream;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

use migration::{Migrator, MigratorTrait};

mod db;
use db::prelude::*;

#[derive(Deserialize)]
struct AddTagRequest {
    tag: String,
}

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
    /*
     * Must specify the associated types.
     * IImageRepository<Entity = Type, PrimaryKey = Type, Model = Type, ActiveModel = Type, UpdateModel = Type, Related = Type, RelatedPrimaryKey = Type>
     */
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
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300)) // 5 minutes
        .max_lifetime(Duration::from_secs(1800)); // 30 minutes

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
    let curdir = std::env::current_dir().unwrap();
    let static_path = curdir.join("wwwroot");
    let images_path = curdir.join("data/images");
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
        .route("/about", get(about))
        .route("/images", get(image_list))
        .route("/images/count", get(image_count))
        .route("/images/{id}", get(image_get))
        .route("/images", post(image_add))
        .route("/images/{id}", put(image_update))
        .route("/images/{id}", delete(image_delete))
        .route("/images/{id}/tags/", get(image_tag_list))
        .route("/images/{id}/tags/", post(image_tag_add))
        .route("/images/{id}/tags/{tag_id}", delete(image_tag_remove))
        .route("/tags/", get(tag_list))
        .route("/tags/count", get(tag_count))
        .route("/tags/{id}", get(tag_get))
        .route("/tags/", post(tag_add))
        .route("/tags/{id}", put(tag_update))
        .route("/tags/{id}", delete(tag_delete))
        .route("/tags/{id}/images/", get(tag_image_list))
        .route("/tags/{id}/images/", post(tag_image_add))
        .route("/tags/{id}/images/{tag_id}", delete(tag_image_remove))
        .nest_service("/assets", ServeDir::new(images_path))
        .fallback_service(ServeDir::new(static_path).append_index_html_on_directories(true))
        .layer(cors)
}

// Handlers
async fn about() -> Result<impl IntoResponse, (StatusCode, String)> {
    let file = tokio::fs::File::open("static/about.md")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(response)
}

async fn image_list(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
) -> Result<Json<ResultSet<ModelWithRelated<ImageModel, TagModel>>>, (StatusCode, String)> {
    match repo.list_with_related(None, None, None).await {
        Ok(images) => Ok(Json(images)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_count(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
) -> Result<Json<u64>, (StatusCode, String)> {
    match repo.count(None).await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_get(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<ModelWithRelated<ImageModel, TagModel>>, (StatusCode, String)> {
    match repo.get_with_related(id).await {
        Ok(Some(image)) => Ok(Json(image)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Image not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_add(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    mut multipart: Multipart,
) -> Result<Json<ImageModel>, (StatusCode, String)> {
    // Read the form data from the multipart fields
    let mut fields = std::collections::HashMap::new();
    let mut image_bytes = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "image_file" {
            // This is the file field
            image_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
            );
        } else {
            // This is a regular form field
            let value = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            fields.insert(name, value);
        }
    }

    // Unwrap the image_bytes and check if it has data
    let image_data =
        image_bytes.ok_or((StatusCode::BAD_REQUEST, "No image provided".to_string()))?;

    if image_data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Image is empty".to_string()));
    }

    // Load image to get dimensions
    let img = ImageReader::new(std::io::Cursor::new(&image_data))
        .with_guessed_format()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid image format: {}", e),
            )
        })?
        .decode()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to decode image: {}", e),
            )
        })?;
    let (width, height) = (img.width(), img.height());
    let images_dir = images_dir();
    fs::create_dir_all(&images_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // start a transaction in case saving the image fails
    let transaction = repo
        .begin_transaction()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mime_type = fields.get("mime_type").cloned().unwrap_or_default();
    let filename = fields.get("filename").cloned().unwrap_or_default();
    let mut extension = if filename.is_empty() {
        None
    } else {
        Path::new(&filename).extension().and_then(|x| x.to_str())
    };

    if extension.is_none() {
        extension = if !mime_type.is_empty() {
            get_mime_extensions_str(&mime_type)
                .and_then(|x| x.first())
                .map(|x| *x)
        } else {
            None
        }
    }

    let extension = extension.unwrap_or("bin");
    let title = fields.get("title").cloned().unwrap_or(filename.clone());
    let alt_text = fields.get("alt_text").cloned().unwrap_or(title.clone());

    // Assign the missing information to the following image model and let the repository create the data record
    let image_model = CreateImageDto {
        title: title,
        description: Some(fields.get("description").cloned().unwrap_or_default()),
        extension: extension.to_string(),
        file_size: image_data.len() as i64,
        mime_type: mime_type,
        width: Some(width as i32),
        height: Some(height as i32),
        alt_text: Some(alt_text),
        tags: Some(fields.get("tags").cloned().unwrap_or_default()),
    };

    let image_model = match repo.create_with_tags(image_model).await {
        Ok(image_model) => image_model,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    // Save the image file
    let filename = format!("{}.{}", image_model.id, extension);
    let file_path = images_dir.join(&filename);
    fs::write(&file_path, &image_data).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save image: {}", e),
        )
    })?;

    // Create thumbnail keeping aspect ratio (max 256px on longest side)
    let thumbnail = img.thumbnail(256, 256);
    let thumb_path = images_dir.join(&get_image_thumb_name(&filename));
    thumbnail.save(&thumb_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save thumbnail: {}", e),
        )
    })?;

    match transaction.commit().await {
        Ok(_) => Ok(Json(image_model)),
        Err(e) => {
            let _ = fs::remove_file(&file_path);
            let _ = fs::remove_file(&thumb_path);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

async fn image_update(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
    Json(image): Json<UpdateImageDto>,
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
    // start a transaction in case saving the image fails
    let transaction = repo
        .begin_transaction()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let image = repo
        .get(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Image not found.".to_string()))?;
    repo.delete_related(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Err(e) = repo.delete(id).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    let images_dir = images_dir();
    let filepath = images_dir.join(format!("{}.{}", id, image.extension));

    if filepath.exists() {
        if let Err(e) = fs::remove_file(&filepath) {
            tracing::warn!("{}", e);
        }
    }

    let thumbpath = get_image_thumb_path(filepath);

    if thumbpath.exists() {
        if let Err(e) = fs::remove_file(&thumbpath) {
            tracing::warn!("{}", e);
        }
    }

    match transaction.commit().await {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_tag_list(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<ResultSet<TagModel>>, (StatusCode, String)> {
    match repo.list_tags(id, None, None).await {
        Ok(tags) => Ok(Json(tags)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn image_tag_add(
    Extension(repo): Extension<Arc<dyn IImageRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
    Json(payload): Json<AddTagRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match repo.add_tags_from_str(id, &payload.tag).await {
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
) -> Result<Json<ResultSet<TagModel>>, (StatusCode, String)> {
    match repo.list(None, None).await {
        Ok(tags) => Ok(Json(tags)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_count(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
) -> Result<Json<u64>, (StatusCode, String)> {
    match repo.count(None).await {
        Ok(count) => Ok(Json(count)),
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

async fn tag_add(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    Json(tag): Json<TagModel>,
) -> Result<Json<TagModel>, (StatusCode, String)> {
    match repo.create(tag).await {
        Ok(created) => Ok(Json(created)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn tag_update(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
    Json(tag): Json<UpdateTagDto>,
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
    let transaction = repo
        .begin_transaction()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    repo.delete_related(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    repo.delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    transaction
        .commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::NO_CONTENT, ()))
}

async fn tag_image_list(
    Extension(repo): Extension<Arc<dyn ITagRepository + Send + Sync>>,
    axum_path(id): axum_path<i64>,
) -> Result<Json<ResultSet<ModelWithRelated<ImageModel, TagModel>>>, (StatusCode, String)> {
    match repo.list_images(id, None, None, None).await {
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

// helper functions
fn images_dir() -> PathBuf {
    let images_env_dir = std::env::var("IMAGES_DIR").unwrap_or("data/images".to_string());
    PathBuf::from(images_env_dir)
}

fn get_image_thumb_name(filename: &str) -> String {
    if filename.is_empty() {
        return filename.to_owned();
    }

    let path = Path::new(filename);
    let base_name = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path.extension().unwrap_or_default().to_string_lossy();
    format!("{}_thumb.{}", base_name, extension)
}

fn get_image_thumb_path<P: AsRef<Path>>(filename: P) -> PathBuf {
    let path = filename.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let base_name = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path.extension().unwrap_or_default().to_string_lossy();
    let thumb_file_name = format!("{}_thumb.{}", base_name, extension);
    parent.join(thumb_file_name)
}

fn parse_i64(s: Option<&String>) -> Option<i64> {
    s.and_then(|v| v.parse::<i64>().ok())
}

fn parse_i32(s: Option<&String>) -> Option<i32> {
    s.and_then(|v| v.parse::<i32>().ok())
}
