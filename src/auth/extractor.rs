use std::sync::Arc;
use axum::{
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum::extract::FromRef;
use serde::Deserialize;
use serde_json::json;
use crate::auth::token::{verify_token, AuthToken};
use crate::app::state::AppState;

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
        println!("State type: {:?}", std::any::type_name::<S>());
        
        // Extract 'token' from query parameters
        let query = Query::<TokenQuery>::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (StatusCode::UNAUTHORIZED, Json(json!({"message": "token missing"}))).into_response()
            })?;
        
        let app_state = Arc::from_ref(state);

        // Verify the token using the app's secret key
        match verify_token(&query.token, &app_state.settings.app.secret_key) {
            Ok(auth_token) => Ok(VerifiedToken(auth_token)),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"message": "invalid api key provided"})),
            ).into_response()),
        }
    }
}