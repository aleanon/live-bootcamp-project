use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{app_state::AppState, domain::auth_api_error::AuthApiError, utils::auth};

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn verify_token(
    State(app_state): State<AppState>,
    Json(token_request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    dbg!(&token_request);

    let banned_token_store = app_state.banned_token_store.read().await;

    let _claims = auth::validate_token(&token_request.token, &*banned_token_store).await?;

    Ok(StatusCode::OK)
}
