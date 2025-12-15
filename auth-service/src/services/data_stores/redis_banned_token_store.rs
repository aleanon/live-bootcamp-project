use std::sync::Arc;

use color_eyre::eyre::eyre;
use redis::{Commands, Connection};
use tokio::sync::Mutex;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    settings::AuthServiceSetting,
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

        let ttl = AuthServiceSetting::load().auth.jwt.time_to_live as u64;
        let mut conn = self.conn.lock().await;
        conn.set_ex(key, true, ttl)
            .map_err(|e| BannedTokenStoreError::DatabaseError(eyre!(e)))
    }

    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let mut conn = self.conn.lock().await;
        conn.exists(&key)
            .map_err(|e| BannedTokenStoreError::DatabaseError(eyre!(e)))
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
