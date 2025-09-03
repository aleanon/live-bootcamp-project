use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::auth_api_error::AuthApiError,
    utils::{auth, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthApiError> {
    let cookie = auth::extract_token(&jar)?;

    let token = cookie.value().to_owned();

    let mut banned_token_store = app_state.banned_token_store.write().await;
    let _claims = auth::validate_token(&token, &*banned_token_store).await?;

    banned_token_store.ban_token(token).await?;

    Ok((jar.remove(JWT_COOKIE_NAME), StatusCode::OK))
}
