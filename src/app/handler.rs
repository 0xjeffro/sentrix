use crate::app::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use reqwest::Response;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

pub async fn proxy_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    println!("Received request from {}: {}", app_state.settings.app.name, payload);
    
    let proxy_start_time = Instant::now();
    
    let response = app_state.http_client
        .post(&app_state.settings.backend.rpc_url)
        .json(&payload)
        .send().await;

    let result = match response {
        Ok(resp) => build_proxy_response(resp).await,
        Err(err) => {
            #[cfg(debug_assertions)]
            eprintln!("Proxy error: {}", err);
            (
                StatusCode::BAD_GATEWAY,
                "Failed to forward request".to_string(),
            ).into_response()
        }
    };
    let proxy_latency = proxy_start_time.elapsed();
    #[cfg(debug_assertions)]
    println!("Proxy latency: {}ms", proxy_latency.as_millis());
    result
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