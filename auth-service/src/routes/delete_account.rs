use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, email::Email},
    utils::{auth, constants::JWT_ELEVATED_COOKIE_NAME},
};

pub async fn delete_account(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthApiError> {
    let elevated_token = auth::extract_token(&jar, JWT_ELEVATED_COOKIE_NAME)?;

    let claims = auth::validate_elevated_auth_token(
        elevated_token,
        &*app_state.banned_token_store.read().await,
    )
    .await?;

    let user_email = Email::try_from(claims.sub)?;

    app_state
        .user_store
        .write()
        .await
        .delete_user(&user_email)
        .await?;

    Ok((jar, StatusCode::NO_CONTENT))
}
