use axum::{extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError,
        data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore},
        email_client::EmailClient,
    },
    settings::Settings,
    utils::auth::{self, create_removal_cookie},
};

#[tracing::instrument(name = "Logout", skip_all, err(Debug))]
pub async fn logout<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    mut jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let config = Settings::load();
    let jwt_cookie_name = config.auth.jwt.cookie_name.clone();
    let jwt_elevated_cookie_name = config.auth.elevated_jwt.cookie_name.clone();

    let token = auth::extract_token(&jar, &jwt_cookie_name)?.to_owned();

    auth::validate_auth_token(&token, &*app_state.banned_token_store.read().await).await?;

    let mut banned_token_store = app_state.banned_token_store.write().await;

    if let Some(cookie) = jar.get(&jwt_elevated_cookie_name) {
        banned_token_store
            .ban_token(cookie.value().to_owned())
            .await?;
        jar = jar.remove(create_removal_cookie(jwt_elevated_cookie_name))
    }

    banned_token_store.ban_token(token).await?;
    jar = jar.remove(create_removal_cookie(jwt_cookie_name));

    Ok((jar, StatusCode::OK))
}
