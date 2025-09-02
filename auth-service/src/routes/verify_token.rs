use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{domain::auth_api_error::AuthApiError, utils::auth};

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn verify_token(
    Json(token_request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    dbg!(&token_request);

    let _claims = auth::validate_token(&token_request.token)?;

    Ok(StatusCode::OK)
}
