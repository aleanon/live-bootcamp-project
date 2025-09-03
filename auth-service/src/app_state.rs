use std::sync::Arc;

use tokio::sync::RwLock;

use crate::domain::data_stores::BannedTokenStore as BannedTokensStoreTrait;
use crate::domain::data_stores::UserStore as UserStoreTrait;

type UserStore = Arc<RwLock<dyn UserStoreTrait>>;
type BannedTokenStore = Arc<RwLock<dyn BannedTokensStoreTrait>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStore,
    pub banned_token_store: BannedTokenStore,
}

impl AppState {
    pub fn new(user_store: UserStore, banned_token_store: BannedTokenStore) -> Self {
        AppState {
            user_store,
            banned_token_store,
        }
    }
}
