use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, login::ValidLoginRequest},
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let login_request = ValidLoginRequest::parse(login_request.email, login_request.password)?;

    let _validated_user = app_state
        .user_store
        .read()
        .await
        .validate_user(login_request.email(), login_request.password())
        .await?;

    Ok(StatusCode::OK)
}
