use std::sync::Arc;

use auth_service::app_state::AppState;
use auth_service::services::data_stores::hashmap_two_fa_code_store::HashMapTwoFaCodeStore;
use auth_service::services::data_stores::mock_email_client::MockEmailClient;
use auth_service::services::data_stores::{PostgresUserStore, RedisBannedTokenStore};
use auth_service::utils::constants::prod;
use auth_service::{Application, configure_postgresql, configure_redis};
use tokio::sync::{Mutex, RwLock};

#[tokio::main]
async fn main() {
    let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
    let pg_pool = configure_postgresql().await;
    let redis_connection = Arc::new(Mutex::new(configure_redis()));
    let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFaCodeStore::default()));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection)));
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}
