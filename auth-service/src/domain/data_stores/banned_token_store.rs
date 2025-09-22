use thiserror::Error;

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Database error: {0}")]
    DatabaseError(#[source] color_eyre::Report),
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}
