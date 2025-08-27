use std::sync::Arc;

use auth_service::{
    app_state::AppState, services::hashmap_user_store::HashMapUserStore, Application,
};
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashMapUserStore::default()));
        let app_state = AppState::new(user_store);

        let app = Application::build(app_state, "127.0.0.0:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        TestApp {
            address,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn login(&self, email: String, password: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            // .json(&format!("email:{},password:{}", email, password))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_2fa(&self, code: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            // .json(&format!("code:{}", code))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_token(&self, token: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            // .json(&format!("token:{}", token))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn delete_account<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/delete-account", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
