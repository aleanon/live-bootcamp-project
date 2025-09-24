use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError,
        data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore},
        email::Email,
        email_client::EmailClient,
    },
    utils::{auth, constants::JWT_ELEVATED_COOKIE_NAME},
};

#[tracing::instrument(name = "Delete Account", skip_all, err(Debug))]
pub async fn delete_account<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
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
