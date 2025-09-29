use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::Deserialize;

use crate::domain::data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore};
use crate::domain::email::Email;
use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, email_client::EmailClient, password::Password},
    utils::{auth, constants::JWT_ELEVATED_COOKIE_NAME},
};

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    new_password: Secret<String>,
}

pub async fn change_password<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    jar: CookieJar,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let token = auth::extract_token(&jar, *JWT_ELEVATED_COOKIE_NAME)?;
    let claim =
        auth::validate_elevated_auth_token(token, &*app_state.banned_token_store.read().await)
            .await?;

    let email = Email::try_from(claim.sub)?;
    let new_password = Password::try_from(request.new_password)?;

    app_state
        .user_store
        .write()
        .await
        .set_new_password(&email, new_password)
        .await?;

    Ok((jar, StatusCode::OK))
}
