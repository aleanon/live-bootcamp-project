use std::sync::Arc;

use auth_service::{
    Application,
    app_state::actual,
    application::get_postgres_pool,
    domain::{
        data_stores::{BannedTokenStore, TwoFaCodeStore},
        email::Email,
    },
    requests::verify_2fa::Verify2FARequest,
    services::data_stores::{
        PostgresUserStore, RedisBannedTokenStore, RedisTwoFaCodeStore,
        hashmap_two_fa_code_store::HashMapTwoFaCodeStore, mock_email_client::MockEmailClient,
    },
    utils::constants::{JWT_COOKIE_NAME, JWT_ELEVATED_COOKIE_NAME, test},
};

use reqwest::{
    Url,
    cookie::{CookieStore, Jar},
};
use secrecy::Secret;
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use testcontainers_modules::{
    postgres,
    redis::Redis,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

type AppState<
    UserStore = PostgresUserStore,
    BannedTokenStore = RedisBannedTokenStore,
    TwoFaCodeStore = HashMapTwoFaCodeStore,
    EmailClient = MockEmailClient,
> = actual::AppState<UserStore, BannedTokenStore, TwoFaCodeStore, EmailClient>;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub two_fa_code_store: Arc<RwLock<dyn TwoFaCodeStore>>,
    pub banned_token_store: Arc<RwLock<dyn BannedTokenStore>>,
    #[allow(unused)]
    user_store_container: ContainerAsync<postgres::Postgres>,
    #[allow(unused)]
    redis_container: ContainerAsync<Redis>,
}

// static TEST_APP_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
// static JANITOR: std::sync::LazyLock<std::sync::RwLock<Janitor>> =
//     std::sync::LazyLock::new(|| std::sync::RwLock::new(Janitor::new()));

impl TestApp {
    pub async fn new() -> Self {
        // TEST_APP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::AcqRel);

        let (redis_container, redis_connection) = setup_and_connect_redis_container().await;
        let redis_connection = Arc::new(Mutex::new(redis_connection));
        let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(
            redis_connection.clone(),
        )));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFaCodeStore::new(redis_connection)));
        let email_client = Arc::new(RwLock::new(MockEmailClient::default()));

        let (user_store_container, pool) = setup_and_connect_user_store_container().await;

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pool)));

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
            user_store_container,
            redis_container,
        }
    }

    pub async fn get_verify_two_fa_request(&self, body: &Value) -> Verify2FARequest {
        let email = Email::try_from(Secret::new(body["email"].as_str().unwrap().to_string()))
            .expect("Failed to parse Email address");

        let (login_attempt_id, code) = self
            .two_fa_code_store
            .read()
            .await
            .get_login_attempt_id_and_two_fa_code(&email)
            .await
            .expect("Failed to get login attempt id and two fa code");

        Verify2FARequest {
            email: email.as_ref().to_owned(),
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

    pub fn get_jwt_token(&self) -> Option<String> {
        let cookie = self
            .cookie_jar
            .cookies(&Url::parse(&self.address).unwrap())?;

        let (_, token) = cookie.to_str().unwrap().split_once('=').unwrap();

        Some(token.to_owned())
    }

    pub fn _get_elevated_jwt_token(&self) -> Option<String> {
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

// static POSTGRES_CONTAINER: OnceCell<RwLock<ContainerAsync<postgres::Postgres>>> =
//     OnceCell::const_new();

// async fn container() -> RwLockReadGuard<'static, ContainerAsync<postgres::Postgres>> {
//     POSTGRES_CONTAINER
//         .get_or_init(async || {
//             let container = postgres::Postgres::default()
//                 .start()
//                 .await
//                 .expect("Failed to start db container");
//             RwLock::new(container)
//         })
//         .await
//         .read()
//         .await
// }

// async fn connect_test_db() -> PgPool {
//     let container = container().await;

//     let db_port = container
//         .get_host_port_ipv4(5432)
//         .await
//         .expect("Failed to get the mapped port of the container");

//     let host = container
//         .get_host()
//         .await
//         .expect("Failed to get the container host address");

//     let db_url = format!("postgres://postgres:postgres@{}:{}", host, db_port);

//     configure_postgresql(db_url).await
// }

// async fn configure_postgresql(postgresql_conn_url: String) -> PgPool {
//     // We are creating a new database for each test case, and we need to ensure each database has a unique name!
//     let db_name = Uuid::new_v4().to_string();

//     configure_database(&postgresql_conn_url, &db_name).await;

//     let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

//     // Create a new connection pool and return it
//     get_postgres_pool(&postgresql_conn_url_with_db)
//         .await
//         .expect("Failed to create Postgres connection pool!")
// }

// async fn configure_database(db_conn_string: &str, db_name: &str) {
//     // Create database connection
//     let connection = PgPoolOptions::new()
//         .connect(db_conn_string)
//         .await
//         .expect("Failed to create Postgres connection pool.");

//     // Create a new database
//     connection
//         .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
//         .await
//         .expect("Failed to create database.");

//     // Connect to new database
//     let db_conn_string = format!("{}/{}", db_conn_string, db_name);

//     let connection = PgPoolOptions::new()
//         .connect(&db_conn_string)
//         .await
//         .expect("Failed to create Postgres connection pool.");

//     // Run migrations against new database
//     sqlx::migrate!()
//         .run(&connection)
//         .await
//         .expect("Failed to migrate the database");
// }

// async fn setup_and_connect_redis_test_db_container() -> Connection {
//     let container = testcontainers_modules::redis::Redis::default().start().await.expect("Failed to start redis test container");

//     let host = container.get_host_port_ipv4(6379)
// }

async fn setup_and_connect_user_store_container() -> (ContainerAsync<postgres::Postgres>, PgPool) {
    let container = postgres::Postgres::default()
        .start()
        .await
        .expect("Failed to start container");

    let db_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get the mapped port of the container");

    let host = container
        .get_host()
        .await
        .expect("Failed to get the container host address");

    let db_url = format!("postgres://postgres:postgres@{}:{}", host, db_port);

    let connection = get_postgres_pool(&db_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");

    (container, connection)
}

async fn setup_and_connect_redis_container() -> (ContainerAsync<Redis>, redis::Connection) {
    let container = Redis::default()
        .start()
        .await
        .expect("Failed to start container");

    let db_port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get the mapped port of the container");

    let host = container
        .get_host()
        .await
        .expect("Failed to get the container host address");

    let db_url = format!("redis://{}:{}/", host, db_port);

    let connection = redis::Client::open(db_url)
        .expect("Failed to open redis client")
        .get_connection()
        .expect("Failed to connect redis client");

    (container, connection)
}
