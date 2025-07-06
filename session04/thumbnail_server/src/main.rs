use anyhow::Result;
use axum::{
    Json, Router,
    http::StatusCode,
    response::{Html, IntoResponse, Json as JsonResponse},
    routing::{get, post},
};
use serde_json::{Value as JsonValue, json};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let app = create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

// Setup the router
fn create_router() -> Router {
    let static_path = std::env::current_dir().unwrap().join("wwwroot");
    Router::new().fallback_service(ServeDir::new(static_path))
}
