use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, user::User},
};

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

pub async fn delete_account(
    State(app_state): State<AppState>,
    Json(request): Json<DeleteRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let user = User::parse(request.email, request.password, request.requires_2fa)?;

    let validated_user = app_state
        .user_store
        .read()
        .await
        .validate_user(user.email(), user.password())
        .await?;

    app_state
        .user_store
        .write()
        .await
        .delete_user(&validated_user)
        .await?;

    Ok((
        StatusCode::OK,
        format!(
            "User {} deleted successfully",
            validated_user.email().as_str()
        ),
    ))
}
