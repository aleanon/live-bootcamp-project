use crate::domain::{email::Email, login_attempt_id::LoginAttemptId, two_fa_code::TwoFaCode};
use thiserror::Error;

#[cfg_attr(debug_assertions, derive(PartialEq))]
#[derive(Debug, Error)]
pub enum TwoFaCodeStoreError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Invalid 2FA code")]
    Invalid2FACode,
    #[error("Unexpected error")]
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait TwoFaCodeStore: Send + Sync {
    async fn store_code(
        &mut self,
        user_id: Email,
        session_id: LoginAttemptId,
        two_fa_code: TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError>;
    async fn validate(
        &self,
        user_id: &Email,
        session_id: &LoginAttemptId,
        two_fa_code: &TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError>;
    async fn delete(&mut self, user_id: &Email) -> Result<(), TwoFaCodeStoreError>;
}
