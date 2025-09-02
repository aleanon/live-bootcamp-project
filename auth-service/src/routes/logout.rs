use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::auth_api_error::AuthApiError,
    utils::{auth, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> Result<impl IntoResponse, AuthApiError> {
    let cookie = auth::extract_token(&jar)?;

    let token = cookie.value().to_owned();

    let _result = auth::validate_token(&token)?;

    Ok((jar.remove(JWT_COOKIE_NAME), StatusCode::OK))
}
