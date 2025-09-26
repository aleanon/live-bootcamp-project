use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError,
        data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore},
        email::Email,
        email_client::EmailClient,
        password::Password,
        user::UserError,
    },
    settings::Settings,
    utils::auth,
};

#[derive(Debug, Deserialize)]
pub struct ElevateRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
}

pub struct ValidElevateRequest {
    email: Email,
    password: Password,
}

impl ValidElevateRequest {
    pub fn parse(elevate_request: ElevateRequest) -> Result<Self, UserError> {
        Ok(ValidElevateRequest {
            email: Email::try_from(elevate_request.email)?,
            password: Password::try_from(elevate_request.password)?,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }
}

#[tracing::instrument(name = "Elevate auth", skip_all, err(Debug))]
pub async fn elevate<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    jar: CookieJar,
    Json(elevate_request): Json<ElevateRequest>,
) -> Result<impl IntoResponse, AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let config = Settings::load();
    let cookie = jar
        .get(&config.auth.jwt.cookie_name)
        .ok_or(AuthApiError::MissingToken)?;

    auth::validate_auth_token(cookie.value(), &*app_state.banned_token_store.read().await).await?;

    let elevate_request = ValidElevateRequest::parse(elevate_request)?;

    app_state
        .user_store
        .read()
        .await
        .authenticate_user(elevate_request.email(), elevate_request.password())
        .await?;

    let elevated_cookie = auth::generate_elevated_auth_cookie(elevate_request.email(), &config)?;

    Ok((jar.add(elevated_cookie), StatusCode::OK))
}
