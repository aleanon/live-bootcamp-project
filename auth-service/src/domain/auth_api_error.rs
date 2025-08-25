use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::user::UserError;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum AuthApiError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(#[from] UserError),
    #[error("Unexpected error")]
    UnexpectedError,
}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AuthApiError::InvalidCredentials(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthApiError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthApiError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status_code, body).into_response()
    }
}
