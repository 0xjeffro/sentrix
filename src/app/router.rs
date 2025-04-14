use std::sync::Arc;
use axum::Router;
use axum::routing::post;
use crate::app::handler::proxy_handler;
use crate::app::state::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(proxy_handler))
        .with_state(state)
}