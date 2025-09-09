use std::collections::HashMap;

use crate::domain::{
    data_stores::{TwoFaCodeStore, TwoFaCodeStoreError},
    email::Email,
    login_attempt_id::LoginAttemptId,
    two_fa_code::TwoFaCode,
};

#[derive(Default)]
pub struct HashMapTwoFaCodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFaCode)>,
}

impl HashMapTwoFaCodeStore {
    pub fn new() -> Self {
        Self {
            codes: HashMap::new(),
        }
    }

    pub fn has_login_attempt_id(&self, attempt_id: &LoginAttemptId) -> bool {
        self.codes
            .values()
            .find(|(id, _)| id == attempt_id)
            .is_some()
    }
}

#[async_trait::async_trait]
impl TwoFaCodeStore for HashMapTwoFaCodeStore {
    async fn store_code(
        &mut self,
        user_id: Email,
        login_attempt_id: LoginAttemptId,
        two_fa_code: TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError> {
        if self.codes.contains_key(&user_id) {
            self.codes.entry(user_id).and_modify(|(id, code)| {
                *id = login_attempt_id;
                *code = two_fa_code;
            });
        } else {
            self.codes.insert(user_id, (login_attempt_id, two_fa_code));
        }
        Ok(())
    }

    async fn validate(
        &self,
        user_id: &Email,
        login_attempt_id: &LoginAttemptId,
        two_fa_code: &TwoFaCode,
    ) -> Result<(), TwoFaCodeStoreError> {
        let Some((id, code)) = self.codes.get(user_id) else {
            return Err(TwoFaCodeStoreError::UserNotFound);
        };

        if id != login_attempt_id {
            return Err(TwoFaCodeStoreError::InvalidAttemptId);
        }
        if code != two_fa_code {
            return Err(TwoFaCodeStoreError::Invalid2FACode);
        }
        Ok(())
    }

    async fn get_login_attempt_id_and_two_fa_code(
        &self,
        user_id: &Email,
    ) -> Result<(LoginAttemptId, TwoFaCode), TwoFaCodeStoreError> {
        let Some((id, code)) = self.codes.get(user_id) else {
            return Err(TwoFaCodeStoreError::UserNotFound);
        };
        Ok((id.clone(), code.clone()))
    }

    async fn delete(&mut self, user_id: &Email) -> Result<(), TwoFaCodeStoreError> {
        self.codes
            .remove(user_id)
            .ok_or(TwoFaCodeStoreError::UserNotFound)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_code() {
        let mut store = HashMapTwoFaCodeStore::new();
        let user_id = Email::try_from("test@example.com".to_owned()).unwrap();
        let session_id = LoginAttemptId::new();
        let two_fa_code = TwoFaCode::new();

        store
            .store_code(user_id.clone(), session_id.clone(), two_fa_code.clone())
            .await
            .unwrap();

        let (id, code) = store.codes.get(&user_id).unwrap();
        assert_eq!(id, &session_id);
        assert_eq!(code, &two_fa_code);
    }

    #[tokio::test]
    async fn should_pass_valid_login_attempt_id_and_code() {
        let mut store = HashMapTwoFaCodeStore::new();
        let user_id = Email::try_from("test@example.com".to_owned()).unwrap();
        let session_id = LoginAttemptId::new();
        let two_fa_code = TwoFaCode::new();

        store
            .store_code(user_id.clone(), session_id.clone(), two_fa_code.clone())
            .await
            .unwrap();

        assert!(
            store
                .validate(&user_id, &session_id, &two_fa_code)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn should_fail_with_invalid_login_attempt_id() {
        let mut store = HashMapTwoFaCodeStore::new();
        let user_id = Email::try_from("test@example.com".to_owned()).unwrap();
        let session_id = LoginAttemptId::new();
        let two_fa_code = TwoFaCode::new();

        store
            .store_code(user_id.clone(), session_id.clone(), two_fa_code.clone())
            .await
            .unwrap();

        assert_eq!(
            store
                .validate(&user_id, &LoginAttemptId::new(), &two_fa_code)
                .await,
            Err(TwoFaCodeStoreError::InvalidAttemptId)
        );
    }

    #[tokio::test]
    async fn should_fail_with_invalid_code() {
        let mut store = HashMapTwoFaCodeStore::new();
        let user_id = Email::try_from("test@example.com".to_owned()).unwrap();
        let session_id = LoginAttemptId::new();
        let two_fa_code = TwoFaCode::new();

        store
            .store_code(user_id.clone(), session_id.clone(), two_fa_code.clone())
            .await
            .unwrap();

        assert_eq!(
            store
                .validate(&user_id, &session_id, &TwoFaCode::new())
                .await,
            Err(TwoFaCodeStoreError::Invalid2FACode)
        );
    }

    #[tokio::test]
    async fn delete_should_pass_with_valid_user_id() {
        let mut store = HashMapTwoFaCodeStore::new();
        let user_id = Email::try_from("test@example.com".to_owned()).unwrap();
        let session_id = LoginAttemptId::new();
        let two_fa_code = TwoFaCode::new();

        store
            .store_code(user_id.clone(), session_id.clone(), two_fa_code.clone())
            .await
            .unwrap();

        store.delete(&user_id).await.unwrap();
        assert!(store.codes.get(&user_id).is_none());
    }
}
