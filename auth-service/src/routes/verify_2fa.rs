use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError, email::Email, login_attempt_id::LoginAttemptId,
        two_fa_code::TwoFaCode,
    },
    utils::auth,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_factor_code: String,
}

pub async fn verify_2fa(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let email = Email::try_from(request.email)?;
    let login_attempt_id = LoginAttemptId::parse(&request.login_attempt_id)?;
    let two_fa_code = TwoFaCode::parse(request.two_factor_code.clone())?;

    let (stored_attempt_id, stored_two_fa_code) = app_state
        .two_fa_code_store
        .read()
        .await
        .get_login_attempt_id_and_two_fa_code(&email)
        .await?;

    if stored_attempt_id != login_attempt_id {
        return Err(AuthApiError::InvalidLoginAttemptId);
    }
    if stored_two_fa_code != two_fa_code {
        return Err(AuthApiError::InvalidTwoFaCode);
    }

    app_state
        .two_fa_code_store
        .write()
        .await
        .delete(&email)
        .await?;

    let auth_cookie = auth::generate_auth_cookie(&email)?;

    let update_jar = jar.add(auth_cookie);

    Ok((update_jar, StatusCode::OK))
}
