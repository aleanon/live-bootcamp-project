use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{CookieJar, cookie::Cookie};

use crate::{
    app_state::AppState,
    domain::auth_api_error::AuthApiError,
    utils::{
        auth,
        constants::{JWT_COOKIE_NAME, JWT_ELEVATED_COOKIE_NAME},
    },
};

pub async fn logout(
    State(app_state): State<AppState>,
    mut jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AuthApiError> {
    let token = auth::extract_token(&jar, JWT_COOKIE_NAME)?.to_owned();

    auth::validate_auth_token(&token, &*app_state.banned_token_store.read().await).await?;

    let mut banned_token_store = app_state.banned_token_store.write().await;

    if let Some(cookie) = jar.get(JWT_ELEVATED_COOKIE_NAME) {
        banned_token_store
            .ban_token(cookie.value().to_owned())
            .await?;
        jar = jar.remove(Cookie::build((JWT_ELEVATED_COOKIE_NAME, "")).build());
    }

    banned_token_store.ban_token(token).await?;
    jar = jar.remove(Cookie::build((JWT_COOKIE_NAME, "")).build());

    Ok((jar, StatusCode::OK))
}
