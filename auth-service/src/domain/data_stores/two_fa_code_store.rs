use crate::domain::{email::Email, two_fa_attempt_id::TwoFaAttemptId, two_fa_code::TwoFaCode};
use thiserror::Error;

#[cfg_attr(debug_assertions, derive(PartialEq))]
#[derive(Debug, Error)]
pub enum TwoFaCodeStoreError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid session")]
    InvalidAttemptId,
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
