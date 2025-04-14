use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use config::Value;
use crate::app::state::AppState;

pub async fn proxy_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    println!("Received request from {}: {}", app_state.settings.app.name, payload);
    "OK"
}