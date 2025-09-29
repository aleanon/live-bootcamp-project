use color_eyre::Report;
use thiserror::Error;

use crate::domain::{
    email::Email,
    password::Password,
    user::{User, ValidatedUser},
};

#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Incorrect password")]
    IncorrectPassword,
    #[error("Unexpected error {0}")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UserAlreadyExists, Self::UserAlreadyExists) => true,
            (Self::UserNotFound, Self::UserNotFound) => true,
            (Self::IncorrectPassword, Self::IncorrectPassword) => true,
            (Self::UnexpectedError(_), Self::UnexpectedError(_)) => true,
            _ => false,
        }
    }
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn set_new_password(
        &mut self,
        email: &Email,
        new_password: Password,
    ) -> Result<(), UserStoreError>;
    async fn authenticate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<ValidatedUser, UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn delete_user(&mut self, user: &Email) -> Result<(), UserStoreError>;
}
