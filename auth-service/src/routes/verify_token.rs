use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError,
        data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore},
        email_client::EmailClient,
    },
    utils::auth,
};

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[tracing::instrument(name = "Verify Token", skip_all, err(Debug))]
pub async fn verify_token<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    Json(token_request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let banned_token_store = app_state.banned_token_store.read().await;

    let _claims = auth::validate_auth_token(&token_request.token, &*banned_token_store).await?;

    Ok(StatusCode::OK)
}
