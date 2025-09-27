use std::sync::Arc;

use color_eyre::eyre::Context;
use redis::Commands;
use secrecy::ExposeSecret;
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
            .map_err(|e| TwoFaCodeStoreError::UnexpectedError(e.into()))?;

        self.client
            .lock()
            .await
            .set_ex(key, value, TEN_MINUTES_IN_SECONDS)
            .map_err(|e| TwoFaCodeStoreError::UnexpectedError(e.into()))
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
                .wrap_err("Failed to serialize 2FA tuple")
                .map_err(TwoFaCodeStoreError::UnexpectedError)?;

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
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref().expose_secret())
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::Secret;
    use std::sync::Arc;
    use testcontainers_modules::{
        redis::Redis,
        testcontainers::{ContainerAsync, runners::AsyncRunner},
    };
    use tokio::sync::Mutex;

    async fn setup_redis_container() -> (ContainerAsync<Redis>, Arc<Mutex<redis::Connection>>) {
        let container = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");

        let host = container
            .get_host()
            .await
            .expect("Failed to get container host");

        let port = container
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis port");

        let redis_url = format!("redis://{}:{}/", host, port);

        let client = redis::Client::open(redis_url).expect("Failed to create Redis client");

        let connection = client.get_connection().expect("Failed to connect to Redis");

        (container, Arc::new(Mutex::new(connection)))
    }

    fn create_test_email() -> Email {
        Email::try_from(Secret::new("test@example.com".to_string())).unwrap()
    }

    #[tokio::test]
    async fn test_store_and_retrieve_code() {
        let (_container, redis_connection) = setup_redis_container().await;
        let mut store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();
        let attempt_id = TwoFaAttemptId::new();
        let code = TwoFaCode::new();

        // Store the code
        let result = store
            .store_code(email.clone(), attempt_id.clone(), code.clone())
            .await;
        assert!(result.is_ok());

        // Retrieve and validate the code
        let result = store.get_login_attempt_id_and_two_fa_code(&email).await;
        assert!(result.is_ok());

        let (retrieved_attempt_id, retrieved_code) = result.unwrap();
        assert_eq!(retrieved_attempt_id, attempt_id);
        assert_eq!(retrieved_code, code);
    }

    #[tokio::test]
    async fn test_validate_success() {
        let (_container, redis_connection) = setup_redis_container().await;
        let mut store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();
        let attempt_id = TwoFaAttemptId::new();
        let code = TwoFaCode::new();

        // Store the code first
        store
            .store_code(email.clone(), attempt_id.clone(), code.clone())
            .await
            .unwrap();

        // Validate should succeed
        let result = store.validate(&email, &attempt_id, &code).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_invalid_attempt_id() {
        let (_container, redis_connection) = setup_redis_container().await;
        let mut store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();
        let stored_attempt_id = TwoFaAttemptId::new();
        let different_attempt_id = TwoFaAttemptId::new();
        let code = TwoFaCode::new();

        // Store with one attempt ID
        store
            .store_code(email.clone(), stored_attempt_id, code.clone())
            .await
            .unwrap();

        // Validate with different attempt ID should fail
        let result = store.validate(&email, &different_attempt_id, &code).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TwoFaCodeStoreError::InvalidAttemptId => {}
            _ => panic!("Expected InvalidAttemptId"),
        }
    }

    #[tokio::test]
    async fn test_validate_invalid_2fa_code() {
        let (_container, redis_connection) = setup_redis_container().await;
        let mut store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();
        let attempt_id = TwoFaAttemptId::new();
        let stored_code = TwoFaCode::new();
        let different_code = TwoFaCode::new();

        // Store with one code
        store
            .store_code(email.clone(), attempt_id.clone(), stored_code)
            .await
            .unwrap();

        // Validate with different code should fail
        let result = store.validate(&email, &attempt_id, &different_code).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TwoFaCodeStoreError::Invalid2FACode => {}
            _ => panic!("Expected Invalid2FACode"),
        }
    }

    #[tokio::test]
    async fn test_validate_user_not_found() {
        let (_container, redis_connection) = setup_redis_container().await;
        let store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();
        let attempt_id = TwoFaAttemptId::new();
        let code = TwoFaCode::new();

        // Validate without storing anything should fail
        let result = store.validate(&email, &attempt_id, &code).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TwoFaCodeStoreError::UserNotFound => {}
            _ => panic!("Expected UserNotFound"),
        }
    }

    #[tokio::test]
    async fn test_get_login_attempt_id_and_two_fa_code_user_not_found() {
        let (_container, redis_connection) = setup_redis_container().await;
        let store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();

        let result = store.get_login_attempt_id_and_two_fa_code(&email).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TwoFaCodeStoreError::UserNotFound => {}
            _ => panic!("Expected UserNotFound"),
        }
    }

    #[tokio::test]
    async fn test_delete_success() {
        let (_container, redis_connection) = setup_redis_container().await;
        let mut store = RedisTwoFaCodeStore::new(redis_connection);

        let email = create_test_email();
        let attempt_id = TwoFaAttemptId::new();
        let code = TwoFaCode::new();

        // Store some data first
        store
            .store_code(email.clone(), attempt_id, code)
            .await
            .unwrap();

        // Verify it exists
        let result = store.get_login_attempt_id_and_two_fa_code(&email).await;
        assert!(result.is_ok());

        // Delete it
        let result = store.delete(&email).await;
        assert!(result.is_ok());

        // Verify it's gone
        let result = store.get_login_attempt_id_and_two_fa_code(&email).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TwoFaCodeStoreError::UserNotFound => {}
            _ => panic!("Expected UserNotFound after deletion"),
        }
    }

    #[tokio::test]
    async fn test_get_key_format() {
        let email = create_test_email();
        let key = get_key(&email);

        assert!(key.starts_with(TWO_FA_CODE_PREFIX));
        assert!(key.contains("test@example.com"));
        assert_eq!(key, format!("{}test@example.com", TWO_FA_CODE_PREFIX));
    }

    #[tokio::test]
    async fn test_multiple_users_isolation() {
        let (_container, redis_connection) = setup_redis_container().await;
        let mut store = RedisTwoFaCodeStore::new(redis_connection);

        let email1 = Email::try_from(Secret::new("user1@example.com".to_string())).unwrap();
        let email2 = Email::try_from(Secret::new("user2@example.com".to_string())).unwrap();

        let attempt_id1 = TwoFaAttemptId::new();
        let attempt_id2 = TwoFaAttemptId::new();
        let code1 = TwoFaCode::new();
        let code2 = TwoFaCode::new();

        // Store data for both users
        store
            .store_code(email1.clone(), attempt_id1.clone(), code1.clone())
            .await
            .unwrap();
        store
            .store_code(email2.clone(), attempt_id2.clone(), code2.clone())
            .await
            .unwrap();

        // Verify each user can only access their own data
        let result1 = store.validate(&email1, &attempt_id1, &code1).await;
        assert!(result1.is_ok());

        let result2 = store.validate(&email2, &attempt_id2, &code2).await;
        assert!(result2.is_ok());

        // Cross-validation should fail
        let cross_result = store.validate(&email1, &attempt_id2, &code2).await;
        assert!(cross_result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (_container, redis_connection) = setup_redis_container().await;
        let store = Arc::new(RedisTwoFaCodeStore::new(redis_connection));

        let email1 = Email::try_from(Secret::new("user1@example.com".to_string())).unwrap();
        let email2 = Email::try_from(Secret::new("user2@example.com".to_string())).unwrap();

        let store1 = Arc::clone(&store);
        let store2 = Arc::clone(&store);

        let handle1 = tokio::spawn(async move {
            let attempt_id = TwoFaAttemptId::new();
            let code = TwoFaCode::new();
            store1.validate(&email1, &attempt_id, &code).await
        });

        let handle2 = tokio::spawn(async move {
            let attempt_id = TwoFaAttemptId::new();
            let code = TwoFaCode::new();
            store2.validate(&email2, &attempt_id, &code).await
        });

        let (result1, result2) = tokio::join!(handle1, handle2);

        // Both should fail with UserNotFound since no data is stored
        assert!(result1.unwrap().is_err());
        assert!(result2.unwrap().is_err());
    }
}
