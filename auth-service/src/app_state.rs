use std::sync::Arc;

use tokio::sync::RwLock;

use crate::domain::data_stores::UserStore as UserStoreTrait;

type UserStore = Arc<RwLock<dyn UserStoreTrait>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStore,
}

impl AppState {
    pub fn new(user_store: UserStore) -> Self {
        AppState { user_store }
    }
}
