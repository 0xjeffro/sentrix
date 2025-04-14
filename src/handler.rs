use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use config::Value;
use crate::config::Settings;

pub async fn proxy_handler(
    State(settings): State<Settings>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    println!(
        "Received request from {}: {}",
        settings.app_name,
        payload);
    "OK"
}