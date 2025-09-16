use std::sync::Arc;

use auth_service::app_state::AppState;
use auth_service::services::hashmap_two_fa_code_store::HashMapTwoFaCodeStore;
use auth_service::services::hashmap_user_store::HashMapUserStore;
use auth_service::services::hashset_banned_token_store::HashSetBannedTokenStore;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::utils::constants::{DATABASE_URL, prod};
use auth_service::{Application, get_postgres_pool};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = Arc::new(RwLock::new(HashMapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashSetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFaCodeStore::default()));
    let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let pg_pool = configure_postgresql().await;

    Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}
