use thiserror::Error;

use crate::domain::{
    email::Email,
    password::Password,
    user::{User, UserError},
};

#[derive(Debug, PartialEq, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Unexpected error")]
    UnexpectedError,
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(UserError),
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<&User, UserStoreError>;
}
