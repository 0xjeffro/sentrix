use crate::app::state::AppState;
use crate::auth::extractor::VerifiedToken;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use reqwest::Response;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub async fn proxy_handler(
    State(app_state): State<Arc<AppState>>,
    VerifiedToken(auth_token): VerifiedToken,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    println!(
        "Received request from {}: {}",
        app_state.settings.app.name, payload
    );

    let start_time = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string();
    info!(
        event = "request_received",
        user = auth_token.user,
        request = payload.to_string(),
        max_qps = auth_token.qps,
        exp = auth_token.exp,
        request_id = request_id
    );

    let response = app_state
        .http_client
        .post(&app_state.settings.backend.rpc_url)
        .json(&payload)
        .send()
        .await;

    info!(
        event = "request_forwarded",
        user = auth_token.user,
        duration = start_time.elapsed().as_secs_f64() * 1000.0,
        backend_url = app_state.settings.backend.rpc_url,
        request_id = request_id
    );

    let result = match response {
        Ok(resp) => build_proxy_response(resp, &request_id).await,
        Err(_err) => {
            #[cfg(debug_assertions)]
            eprintln!("Proxy error: {}", _err);
            (
                StatusCode::BAD_GATEWAY,
                "Failed to forward request".to_string(),
            )
                .into_response()
        }
    };
    info!(
        event = "response_sent",
        user = auth_token.user,
        result = format!("{:?}", result),
        duration = start_time.elapsed().as_secs_f64() * 1000.0,
        request_id = request_id
    );
    result
}

async fn build_proxy_response(resp: Response, request_id: &str) -> axum::response::Response {
    let status = resp.status();

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/json")
        .to_string();
    match resp.bytes().await {
        Ok(body) => {
            info!(
                event = "response_body",
                body = String::from_utf8_lossy(&body).to_string(),
                status = status.to_string(),
                content_type = content_type,
                request_id = request_id,
            );
            (
                status,
                [(axum::http::header::CONTENT_TYPE, content_type)],
                body,
            )
                .into_response()
        }

        Err(_err) => {
            #[cfg(debug_assertions)]
            eprintln!("Failed to read response body: {}: {}", status, _err);

            let fallback_body = "Failed to read response body";
            info!(
                event = "prepare_response",
                body = fallback_body,
                status = status.to_string(),
                content_type = content_type,
                request_id = request_id,
            );
            (StatusCode::BAD_GATEWAY, fallback_body.to_string()).into_response()
        }
    }
}
