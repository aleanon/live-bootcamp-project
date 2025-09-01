use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::utils::auth::GenerateTokenError;

use super::{data_stores::UserStoreError, user::UserError};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum AuthApiError {
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(#[from] UserError),
    #[error("Authentication failed: {0}")]
    AuthenticationError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Unexpected error")]
    UnexpectedError,
}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AuthApiError::UserNotFound => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthApiError::InvalidCredentials(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AuthApiError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthApiError::AuthenticationError(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthApiError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status_code, body).into_response()
    }
}

impl From<UserStoreError> for AuthApiError {
    fn from(error: UserStoreError) -> Self {
        match error {
            UserStoreError::UserAlreadyExists => AuthApiError::UserAlreadyExists,
            UserStoreError::UnexpectedError => AuthApiError::UnexpectedError,
            UserStoreError::UserNotFound => AuthApiError::UserNotFound,
            UserStoreError::IncorrectPassword => AuthApiError::AuthenticationError(Box::new(error)),
        }
    }
}

impl From<GenerateTokenError> for AuthApiError {
    fn from(error: GenerateTokenError) -> Self {
        match error {
            GenerateTokenError::TokenError(err) => AuthApiError::AuthenticationError(Box::new(err)),
            GenerateTokenError::UnexpectedError => AuthApiError::UnexpectedError,
        }
    }
}
