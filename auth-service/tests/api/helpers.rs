use std::sync::Arc;

use auth_service::{
    Application,
    app_state::AppState,
    domain::{data_stores::TwoFaCodeStore, email::Email},
    requests::verify_2fa::Verify2FARequest,
    services::{
        hashmap_two_fa_code_store::HashMapTwoFaCodeStore, hashmap_user_store::HashMapUserStore,
        hashset_banned_token_store::HashSetBannedTokenStore, mock_email_client::MockEmailClient,
    },
    utils::constants::{JWT_COOKIE_NAME, JWT_ELEVATED_COOKIE_NAME, test},
};

use reqwest::{
    Url,
    cookie::{CookieStore, Jar},
};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub two_fa_code_store: Arc<RwLock<HashMapTwoFaCodeStore>>,
    pub banned_token_store: Arc<RwLock<HashSetBannedTokenStore>>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashMapUserStore::default()));
        let banned_token_store = Arc::new(RwLock::new(HashSetBannedTokenStore::default()));
        let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFaCodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient::default()));

        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client,
        );

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
            two_fa_code_store,
            banned_token_store,
        }
    }

    pub async fn get_verify_two_fa_request(&self, body: &Value) -> Verify2FARequest {
        let email = Email::try_from(body["email"].as_str().unwrap().to_string())
            .expect("Failed to parse Email address");

        let (login_attempt_id, code) = self
            .two_fa_code_store
            .read()
            .await
            .get_login_attempt_id_and_two_fa_code(&email)
            .await
            .expect("Failed to get login attempt id and two fa code");

        Verify2FARequest {
            email: email.as_ref().to_string(),
            login_attempt_id: login_attempt_id.to_string(),
            two_factor_code: code.to_string(),
        }
    }

    pub fn add_invalid_cookie(&self) {
        self.cookie_jar.add_cookie_str(
            &format!(
                "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
                JWT_COOKIE_NAME
            ),
            &Url::parse(&self.address).expect("Failed to parse URL"),
        );
    }

    pub fn get_jwt_token(&self) -> String {
        let cookie = self
            .cookie_jar
            .cookies(&Url::parse(&self.address).unwrap())
            .unwrap();

        let (_, token) = cookie.to_str().unwrap().split_once('=').unwrap();

        token.to_owned()
    }

    pub fn get_elevated_jwt_token(&self) -> Option<String> {
        self.cookie_jar
            .cookies(&Url::parse(&self.address).unwrap())?
            .to_str()
            .expect("Unable to make cookie into &str")
            .split(';')
            .map(|c| c.trim())
            .find(|c| c.starts_with(JWT_ELEVATED_COOKIE_NAME))
            .and_then(|c| {
                c.split_once('=')
                    .and_then(|(_, token)| Some(token.to_owned()))
            })
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

    pub async fn verify_2fa<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_token<Body: Serialize>(&self, token: &Body) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(token)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn delete_account(&self) -> reqwest::Response {
        self.http_client
            .delete(&format!("{}/delete-account", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_elevate<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/elevate", &self.address))
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
