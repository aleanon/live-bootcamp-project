use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::utils::auth::TokenAuthError;

use super::{
    data_stores::{BannedTokenStoreError, TwoFaCodeStoreError, UserStoreError},
    email_client::EmailClientError,
    two_fa_error::TwoFaError,
    user::UserError,
};

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
    #[error("Invalid input: {0}")]
    InvalidInput(Box<dyn std::error::Error + Send + Sync>),
    #[error("Missing token")]
    MissingToken,
    #[error("Authentication failed: {0}")]
    AuthenticationError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Invalid login attempt ID")]
    InvalidLoginAttemptId,
    #[error("Invalid two-factor authentication code")]
    InvalidTwoFaCode,
    #[error("Unexpected error")]
    UnexpectedError,
}

impl IntoResponse for AuthApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AuthApiError::InvalidInput(_) | AuthApiError::MissingToken => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            AuthApiError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AuthApiError::AuthenticationError(_)
            | AuthApiError::UserNotFound
            | AuthApiError::InvalidLoginAttemptId
            | AuthApiError::InvalidTwoFaCode => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthApiError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status_code, body).into_response()
    }
}

impl From<UserError> for AuthApiError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::InvalidEmail | UserError::InvalidPassword => {
                AuthApiError::InvalidInput(Box::new(error))
            }
        }
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

impl From<TokenAuthError> for AuthApiError {
    fn from(error: TokenAuthError) -> Self {
        match error {
            TokenAuthError::InvalidToken
            | TokenAuthError::TokenError(_)
            | TokenAuthError::TokenIsBanned => AuthApiError::AuthenticationError(Box::new(error)),
            TokenAuthError::MissingToken => AuthApiError::MissingToken,
            TokenAuthError::UnexpectedError => AuthApiError::UnexpectedError,
        }
    }
}

impl From<BannedTokenStoreError> for AuthApiError {
    fn from(error: BannedTokenStoreError) -> Self {
        match error {
            BannedTokenStoreError::DatabaseError(_) => AuthApiError::UnexpectedError,
        }
    }
}

impl From<TwoFaCodeStoreError> for AuthApiError {
    fn from(error: TwoFaCodeStoreError) -> Self {
        match error {
            TwoFaCodeStoreError::UnexpectedError => AuthApiError::UnexpectedError,
            TwoFaCodeStoreError::UserNotFound => AuthApiError::UserNotFound,
            TwoFaCodeStoreError::InvalidAttemptId | TwoFaCodeStoreError::Invalid2FACode => {
                AuthApiError::AuthenticationError(Box::new(error))
            }
        }
    }
}

impl From<EmailClientError> for AuthApiError {
    fn from(error: EmailClientError) -> Self {
        match error {
            EmailClientError::UnexpectedError => AuthApiError::UnexpectedError,
        }
    }
}

impl From<TwoFaError> for AuthApiError {
    fn from(error: TwoFaError) -> Self {
        match error {
            TwoFaError::InvalidTwoFaCode | TwoFaError::InvalidLoginAttemptID => {
                AuthApiError::InvalidInput(Box::new(error))
            }
        }
    }
}
