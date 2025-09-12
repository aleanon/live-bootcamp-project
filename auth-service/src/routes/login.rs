use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError, email::Email, two_fa_attempt_id::TwoFaAttemptId,
        two_fa_code::TwoFaCode, user::ValidatedUser,
    },
    requests::login::{LoginRequest, ValidLoginRequest},
    responses::login::{LoginResponse, TwoFactorAuthResponse},
    utils::auth::generate_auth_cookie,
};

pub async fn login(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let login_request = ValidLoginRequest::parse(login_request.email, login_request.password)?;

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

async fn handle_2fa(
    email: Email,
    app_state: AppState,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthApiError> {
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
        .read()
        .await
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
