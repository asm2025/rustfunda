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
    let static_path = std::env::current_dir().unwrap().join("src/www");
    Router::new()
        .route("/html", get(get_html))
        .route("/json", get(get_json))
        .route("/post", post(post_json))
        .fallback_service(ServeDir::new(static_path))
}

async fn get_html() -> Html<String> {
    let content = "<p>Hello, <strong>World!</strong></p>".to_string();
    Html(content)
}

async fn get_json() -> Json<JsonValue> {
    let data = json!({
        "message": "Hello, JSON!",
        "status": "success"
    });
    Json(data)
}

async fn post_json(payload: Json<JsonValue>) -> impl IntoResponse {
    // Extract the JSON value from the payload
    let json_data = payload.0;

    // Validate the JSON
    match validate_json(&json_data) {
        Ok(validated_json) => {
            // Process the validated JSON here
            println!("Received valid JSON: {}", validated_json);

            // Create a response with the validated data
            let response = json!({
                "status": "success",
                "message": "JSON validated successfully",
                "data": validated_json
            });

            (StatusCode::OK, JsonResponse(response))
        }
        Err(e) => {
            // Return error response
            let error_response = json!({
                "status": "error",
                "message": format!("Validation failed: {}", e)
            });

            (StatusCode::BAD_REQUEST, JsonResponse(error_response))
        }
    }
}

fn validate_json(json_input: &JsonValue) -> Result<JsonValue, ValidationError> {
    match serde_json::to_string(json_input) {
        Ok(json_string) => match serde_json::from_str(&json_string) {
            Ok(validated) => Ok(validated),
            Err(e) => Err(ValidationError::from(e)),
        },
        Err(e) => Err(ValidationError::InvalidJson(e.to_string())),
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidJson(String),
    UnsupportedType(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidationError::InvalidJson(msg) => write!(f, "Invalid JSON format: {}", msg),
            ValidationError::UnsupportedType(msg) => write!(f, "Unsupported input type: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<serde_json::Error> for ValidationError {
    fn from(error: serde_json::Error) -> Self {
        ValidationError::InvalidJson(error.to_string())
    }
}
