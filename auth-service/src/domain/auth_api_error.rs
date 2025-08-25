use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug)]
pub enum AuthApiError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
}

impl std::fmt::Display for AuthApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthApiError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthApiError::UserAlreadyExists => write!(f, "User already exists"),
            AuthApiError::UnexpectedError => write!(f, "Internal server error"),
        }
    }
}

impl std::error::Error for AuthApiError {}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AuthApiError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthApiError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthApiError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status_code, body).into_response()
    }
}
