use std::sync::Arc;
use axum::extract::State;
use axum::http::{StatusCode};
use axum::http::header::CONTENT_TYPE;
use axum::Json;
use reqwest::Response;
use axum::response::IntoResponse;
use serde_json::{to_string, Value};
use crate::app::state::AppState;

pub async fn proxy_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    println!("Received request from {}: {}", app_state.settings.app.name, payload);

    let response = app_state.http_client
        .post(&app_state.settings.backend.rpc_url)
        .json(&payload)
        .send().await;

    match response {
        Ok(resp) => build_proxy_response(resp).await,
        Err(err) => {
            #[cfg(debug_assertions)]
            eprintln!("Proxy error: {}", err);
            (
                StatusCode::BAD_GATEWAY,
                "Failed to forward request".to_string(),
            ).into_response()
        }
    }
}

async fn build_proxy_response(resp: Response) -> axum::response::Response {
    let status = resp.status();
    
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/json")
        .to_string();
    
    match resp.bytes().await {
        Ok(body) => (
            status,
            [(axum::http::header::CONTENT_TYPE, content_type)],
            body,
        ).into_response(),

        Err(err) => {
            #[cfg(debug_assertions)]
            eprintln!("Failed to read response body: {}: {}", status, err);
            (
                StatusCode::BAD_GATEWAY,
                "Failed to read backend response".to_string(),
            ).into_response()
        }
    }
}