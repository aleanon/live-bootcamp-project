use std::sync::Arc;

use auth_service::{
    app_state::AppState,
    services::hashmap_user_store::HashMapUserStore,
    utils::constants::{test, JWT_COOKIE_NAME},
    Application,
};

use reqwest::{cookie::Jar, Url};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashMapUserStore::default()));
        let app_state = AppState::new(user_store);

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to build client");

        TestApp {
            address,
            cookie_jar,
            http_client,
        }
    }

    pub fn add_invalid_cookie(&self) {
        self.cookie_jar.add_cookie_str(
            &format!(
                "{}=invalid; HttpOnly; SameSite=Lax; Path=/",
                JWT_COOKIE_NAME
            ),
            &Url::parse(&self.address).expect("Failed to parse URL"),
        );
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

    pub async fn login<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
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

    pub async fn verify_2fa(&self, _code: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            // .json(&format!("code:{}", code))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_token(&self, _token: String) -> reqwest::Response {
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

pub fn get_standard_test_user(two_fa: bool) -> Value {
    serde_json::json!({
        "email": "test@example.com",
        "password": "password",
        "requires2FA": two_fa
    })
}
