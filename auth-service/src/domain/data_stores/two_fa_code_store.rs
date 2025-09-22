use crate::domain::{email::Email, two_fa_attempt_id::TwoFaAttemptId, two_fa_code::TwoFaCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TwoFaCodeStoreError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid session")]
    InvalidAttemptId,
    #[error("Invalid 2FA code")]
    Invalid2FACode,
    #[error("Unexpected error")]
    UnexpectedError(#[source] color_eyre::Report),
}

#[cfg(debug_assertions)]
impl PartialEq for TwoFaCodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UserNotFound, Self::UserNotFound) => true,
            (Self::InvalidAttemptId, Self::InvalidAttemptId) => true,
            (Self::Invalid2FACode, Self::Invalid2FACode) => true,
            (Self::UnexpectedError(_), Self::UnexpectedError(_)) => true,
            _ => false,
        }
    }
}

#[async_trait::async_trait]
pub trait TwoFaCodeStore: Send + Sync {
    async fn store_code(
        &mut self,
        user_id: Email,
        login_attempt_id: TwoFaAttemptId,
        two_fa_code: TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError>;

    async fn validate(
        &self,
        user_id: &Email,
        login_attempt_id: &TwoFaAttemptId,
        two_fa_code: &TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError>;

    async fn get_login_attempt_id_and_two_fa_code(
        &self,
        user_id: &Email,
    ) -> Result<(TwoFaAttemptId, TwoFaCode), TwoFaCodeStoreError>;

    async fn delete(&mut self, user_id: &Email) -> Result<(), TwoFaCodeStoreError>;
}
