use std::sync::Arc;

use auth_service::auth_service::{AuthService, configure_postgresql, configure_redis};
use auth_service::services::data_stores::{
    PostgresUserStore, RedisBannedTokenStore, RedisTwoFaCodeStore,
};
use auth_service::services::postmark_email_client::configure_postmark_email_client;
use auth_service::settings::AuthServiceSetting;
use auth_service::utils::constants::prod;
use auth_service::utils::tracing::init_tracing;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");
    let settings = AuthServiceSetting::load();

    let pg_pool = configure_postgresql().await;
    let user_store = PostgresUserStore::new(pg_pool);

    let redis_connection = Arc::new(Mutex::new(configure_redis()));
    let two_fa_code_store = RedisTwoFaCodeStore::new(redis_connection.clone());
    let banned_token_store = RedisBannedTokenStore::new(redis_connection);

    let email_client = configure_postmark_email_client();

    let listener = TcpListener::bind(prod::APP_ADDRESS)
        .await
        .expect("Failed to bind to ip address");

    AuthService::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    )
    .as_standalone(listener, Some(settings.auth.allowed_origins.clone()))
    .await
    .expect("Failed to start application");
}
