use std::sync::Arc;

use auth_service::app_state::AppState;
use auth_service::services::hashmap_user_store::HashMapUserStore;
use auth_service::Application;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = Arc::new(RwLock::new(HashMapUserStore::new()));
    let app_state = AppState::new(user_store);

    Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}
