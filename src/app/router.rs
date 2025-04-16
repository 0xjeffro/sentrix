use crate::app::handler::proxy_handler;
use crate::app::state::AppState;
use axum::Router;
use axum::routing::post;
use std::sync::Arc;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(proxy_handler))
        .with_state(state)
}
