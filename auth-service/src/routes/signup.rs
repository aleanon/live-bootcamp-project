use axum::{extract::FromRequest, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    #[serde(rename = "requires2FA")]
    requires_2fa: bool,
}

pub async fn signup(Json(request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
