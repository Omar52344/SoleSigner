use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_yaml;
use serde_json;

const OPENAPI_YAML: &str = include_str!("../../openapi.yaml");

pub async fn serve_openapi() -> impl IntoResponse {
    match serde_yaml::from_str::<serde_json::Value>(OPENAPI_YAML) {
        Ok(json) => (StatusCode::OK, Json(json)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to parse OpenAPI spec: {}", e) })),
        ),
    }
}

pub async fn serve_openapi_yaml() -> impl IntoResponse {
    (StatusCode::OK, OPENAPI_YAML)
}