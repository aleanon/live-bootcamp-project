// use thiserror::Error;

// use crate::domain::email::Email;

// #[derive(Debug, Error)]
// pub enum TwoFactorStoreError {
//     #[error("2fa not found for user {0:?}")]
//     NotFound(Email),
//     #[error("wrong one-time code")]
//     WrongOneTimeCode,
// }

// #[async_trait::async_trait]
// pub trait TwoFactorStore {
//     async fn create(&mut self, user_id: &Email, secret: &str) -> Result<(), TwoFactorStoreError>;
//     async fn delete(&mut self, user_id: &Email) -> Result<(), TwoFactorStoreError>;
//     async fn validate(&self, user_id: &Email) -> Result<(), TwoFactorStoreError>;
// }
