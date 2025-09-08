use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Serialize;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError, email::Email, login_attempt_id::LoginAttemptId,
        two_fa_code::TwoFaCode,
    },
    utils::auth,
};

#[derive(Debug, Serialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_factor_code: String,
}

pub async fn verify_2fa(
    State(app_state): State<AppState>,
    Json(request): Json<Verify2FARequest>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AuthApiError> {
    let email = Email::try_from(request.email)?;
    let login_attempt_id = LoginAttemptId::parse(&request.login_attempt_id).map_err(
        )),
    )?;
    let two_factor_code = TwoFaCode::parse(request.two_factor_code.clone())?;

    app_state
        .two_fa_code_store
        .read()
        .await
        .validate(&email, &login_attempt_id, &two_factor_code)
        .await?;

    let auth_cookie = auth::generate_auth_cookie(&email)?;

    let update_jar = jar.add(auth_cookie);

    Ok((update_jar, StatusCode::OK))
}
