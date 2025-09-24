use std::sync::Arc;

use auth_service::app_state::AppState;
use auth_service::application::{Application, configure_postgresql, configure_redis};
use auth_service::services::data_stores::{
    PostgresUserStore, RedisBannedTokenStore, RedisTwoFaCodeStore,
};
use auth_service::services::postmark_email_client::configure_postmark_email_client;
use auth_service::utils::constants::prod;
use auth_service::utils::tracing::init_tracing;
use tokio::sync::{Mutex, RwLock};

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    let pg_pool = configure_postgresql().await;
    let redis_connection = Arc::new(Mutex::new(configure_redis()));

    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFaCodeStore::new(
        redis_connection.clone(),
    )));

    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection)));
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let email_client = Arc::new(configure_postmark_email_client());

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
