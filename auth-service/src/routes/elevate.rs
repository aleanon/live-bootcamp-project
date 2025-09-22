use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, data_stores::UserStore},
    requests::elevate::{ElevateRequest, ValidElevateRequest},
    utils::{auth, constants::JWT_COOKIE_NAME},
};

#[tracing::instrument(name = "Elevate auth", skip_all, err(Debug))]
pub async fn elevate(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Json(elevate_request): Json<ElevateRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let cookie = jar.get(JWT_COOKIE_NAME).ok_or(AuthApiError::MissingToken)?;

    auth::validate_auth_token(cookie.value(), &*app_state.banned_token_store.read().await).await?;

    let elevate_request = ValidElevateRequest::parse(elevate_request)?;

    app_state
        .user_store
        .read()
        .await
        .authenticate_user(elevate_request.email(), elevate_request.password())
        .await?;

    let elevated_cookie = auth::generate_elevated_auth_cookie(elevate_request.email())?;

    Ok((jar.add(elevated_cookie), StatusCode::OK))
}
