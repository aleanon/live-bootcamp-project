use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    app_state::AppState, domain::auth_api_error::AuthApiError,
    requests::verify_token::VerifyTokenRequest, utils::auth,
};

#[tracing::instrument(name = "Verify Token", skip_all, err(Debug))]
pub async fn verify_token(
    State(app_state): State<AppState>,
    Json(token_request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let banned_token_store = app_state.banned_token_store.read().await;

    let _claims = auth::validate_auth_token(&token_request.token, &*banned_token_store).await?;

    Ok(StatusCode::OK)
}
