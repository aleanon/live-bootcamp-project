use axum::http::HeaderValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AllowedOriginsStoreError {
    #[error("Unexpected error")]
    UnexpectedError,
}

pub trait AllowedOriginsStore: Send + Sync {
    fn contains(&self, origin: HeaderValue) -> bool;
    fn add_allowed_origin(&self, origin: HeaderValue) -> Result<(), String>;
    fn remove_allowed_origin(&self, origin: HeaderValue) -> Result<(), String>;
}
