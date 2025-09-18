use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::Mutex;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    conn: Arc<Mutex<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn: conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token);

        let ttl = TOKEN_TTL_SECONDS.cast_unsigned();
        let mut conn = self.conn.lock().await;
        conn.set_ex(key, true, ttl)
            .map_err(|e| BannedTokenStoreError::DatabaseError(e.to_string()))
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let mut conn = self.conn.lock().await;
        conn.exists(&key)
            .map_err(|e| BannedTokenStoreError::DatabaseError(e.to_string()))
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
