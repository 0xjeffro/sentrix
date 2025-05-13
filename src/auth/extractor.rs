use crate::app::state::AppState;
use crate::auth::token::{AuthToken, verify_token};
use axum::extract::FromRef;
use axum::{
    Json,
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug)]
pub struct VerifiedToken(pub AuthToken);

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
}

impl<S> FromRequestParts<S> for VerifiedToken
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract 'token' from query parameters
        let query = Query::<TokenQuery>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"message": "token missing"})),
                )
                    .into_response()
            })?;

        let app_state = Arc::from_ref(state);

        // Verify the token using the app's secret key
        match verify_token(&query.token, &app_state.settings.app.secret_key) {
            Ok(auth_token) => {
                let user_id = &auth_token.user;
                let max_qps = &auth_token.qps;
                let expiration = auth_token.exp;
                let now = chrono::Utc::now().timestamp() as u64;
                if expiration < now {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"message": "token expired"})),
                    )
                        .into_response());
                }
                if !app_state.update_and_check_rate_limit(user_id, *max_qps) {
                    return Err((
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(json!({"message": "rate limit exceeded"})),
                    )
                        .into_response());
                }
                Ok(VerifiedToken(auth_token))
            }
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"message": "invalid api key provided"})),
            )
                .into_response()),
        }
    }
}
