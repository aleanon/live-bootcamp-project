use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError,
        data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore},
        email::Email,
        email_client::EmailClient,
        password::Password,
        two_fa_attempt_id::TwoFaAttemptId,
        two_fa_code::TwoFaCode,
        user::{UserError, ValidatedUser},
    },
    utils::auth::generate_auth_cookie,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
}

#[derive(Debug)]
pub struct ValidLoginRequest {
    email: Email,
    password: Password,
}

impl ValidLoginRequest {
    pub fn parse(login_request: LoginRequest) -> Result<Self, UserError> {
        Ok(Self {
            email: Email::try_from(login_request.email)?,
            password: Password::try_from(login_request.password)?,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    // pub fn password_matches(&self, password: &Password) -> bool {
    //     &self.password == password
    // }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub attempt_id: String,
}

#[tracing::instrument(name = "Login", skip_all, err(Debug))]
pub async fn login<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    jar: CookieJar,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let login_request = ValidLoginRequest::parse(login_request)?;

    let validated_user = app_state
        .user_store
        .read()
        .await
        .authenticate_user(login_request.email(), login_request.password())
        .await?;

    match validated_user {
        ValidatedUser::Requires2Fa(email) => handle_2fa(email, app_state, jar).await,
        ValidatedUser::No2Fa(email) => handle_no_2fa(&email, jar).await,
    }
}

async fn handle_2fa<U, B, T, E>(
    email: Email,
    app_state: AppState<U, B, T, E>,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let login_attempt_id = TwoFaAttemptId::new();
    let code = TwoFaCode::new();

    app_state
        .two_fa_code_store
        .write()
        .await
        .store_code(email.clone(), login_attempt_id.clone(), code.clone())
        .await?;

    app_state
        .email_client
        .send_email(&email, "2FA Code", code.as_str())
        .await?;

    let two_factor_auth_response = TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        attempt_id: login_attempt_id.to_string(),
    };

    Ok((
        jar,
        (
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(two_factor_auth_response)),
        ),
    ))
}

async fn handle_no_2fa(
    email: &Email,
    mut jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthApiError> {
    let auth_cookie = generate_auth_cookie(email)?;

    jar = jar.add(auth_cookie);

    Ok((jar, (StatusCode::OK, Json(LoginResponse::RegularAuth))))
}
