use std::sync::Arc;

use color_eyre::eyre::eyre;
use redis::Commands;
use tokio::sync::Mutex;

use crate::domain::{
    data_stores::{TwoFaCodeStore, TwoFaCodeStoreError},
    email::Email,
    two_fa_attempt_id::TwoFaAttemptId,
    two_fa_code::TwoFaCode,
};

pub struct RedisTwoFaCodeStore {
    client: Arc<Mutex<redis::Connection>>,
}

impl RedisTwoFaCodeStore {
    pub fn new(client: Arc<Mutex<redis::Connection>>) -> Self {
        Self { client: client }
    }
}

#[async_trait::async_trait]
impl TwoFaCodeStore for RedisTwoFaCodeStore {
    async fn store_code(
        &mut self,
        user_id: Email,
        login_attempt_id: TwoFaAttemptId,
        two_fa_code: TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError> {
        let key = get_key(&user_id);

        let value = serde_json::to_string(&(login_attempt_id, two_fa_code))
            .map_err(|e| TwoFaCodeStoreError::UnexpectedError(eyre!(e)))?;

        self.client
            .lock()
            .await
            .set_ex(key, value, TEN_MINUTES_IN_SECONDS)
            .map_err(|e| TwoFaCodeStoreError::UnexpectedError(eyre!(e)))
    }

    async fn validate(
        &self,
        user_id: &Email,
        login_attempt_id: &TwoFaAttemptId,
        two_fa_code: &TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError> {
        let (stored_login_attempt_id, stored_two_fa_code) =
            self.get_login_attempt_id_and_two_fa_code(user_id).await?;

        if stored_login_attempt_id != *login_attempt_id {
            return Err(TwoFaCodeStoreError::InvalidAttemptId);
        }
        if stored_two_fa_code != *two_fa_code {
            return Err(TwoFaCodeStoreError::Invalid2FACode);
        }

        Ok(())
    }

    async fn get_login_attempt_id_and_two_fa_code(
        &self,
        user_id: &Email,
    ) -> Result<(TwoFaAttemptId, TwoFaCode), TwoFaCodeStoreError> {
        let key = get_key(&user_id);

        let json_value: String = self
            .client
            .lock()
            .await
            .get(key)
            .map_err(|_| TwoFaCodeStoreError::UserNotFound)?;

        let (login_attempt_id, two_fa_code): (TwoFaAttemptId, TwoFaCode) =
            serde_json::from_str(&json_value)
                .map_err(|e| TwoFaCodeStoreError::UnexpectedError(eyre!(e)))?;

        Ok((login_attempt_id, two_fa_code))
    }

    async fn delete(&mut self, user_id: &Email) -> Result<(), TwoFaCodeStoreError> {
        let key = get_key(&user_id);

        self.client
            .lock()
            .await
            .del(key)
            .map_err(|_| TwoFaCodeStoreError::UserNotFound)
    }
}

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
