use super::auth_service_state::AuthServiceState;
use super::routes::{delete_account, elevate, login, logout, signup, verify_token, verify_two_fa};
use axum::{
    Router,
    http::{HeaderValue, Method, request},
    routing::{delete, post},
};
use redis::{Client, RedisResult};
use secrecy::ExposeSecret;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
};

use crate::domain::data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore};
use crate::domain::email_client::EmailClient;
use crate::routes::{change_password, verify_elevated_token};
use crate::settings::{AllowedOrigins, AuthServiceSetting};
use crate::utils::tracing::{make_span_with_request_id, on_request, on_response};

pub struct AuthService {
    router: Router,
}

impl AuthService {
    pub fn new<U, B, T, E>(
        user_store: U,
        banned_token_store: B,
        two_fa_code_store: T,
        email_client: E,
    ) -> Self
    where
        U: UserStore + 'static,
        B: BannedTokenStore + 'static,
        T: TwoFaCodeStore + 'static,
        E: EmailClient + 'static,
    {
        let state = AuthServiceState::new(
            Arc::new(RwLock::new(user_store)),
            Arc::new(RwLock::new(banned_token_store)),
            Arc::new(RwLock::new(two_fa_code_store)),
            Arc::new(email_client),
        );

        Self::with_state(state)
    }

    pub fn with_state<U, B, T, E>(state: AuthServiceState<U, B, T, E>) -> Self
    where
        U: UserStore + 'static,
        B: BannedTokenStore + 'static,
        T: TwoFaCodeStore + 'static,
        E: EmailClient + 'static,
    {
        let router = Router::new()
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_two_fa))
            .route("/verify-token", post(verify_token))
            .route("/verify-elevated-token", post(verify_elevated_token))
            .route("/elevate", post(elevate))
            .route("/change-password", post(change_password))
            .route("/delete-account", delete(delete_account))
            .fallback_service(ServeDir::new("assets"))
            .with_state(state);

        AuthService { router }
    }

    fn with_trace_layer(mut self) -> Self {
        self.router = self.router.layer(
            TraceLayer::new_for_http()
                .make_span_with(make_span_with_request_id)
                .on_request(on_request)
                .on_response(on_response),
        );
        self
    }

    pub fn as_nested_router(mut self, allow_origin: Option<AllowedOrigins>) -> Router {
        if let Some(allowed_origin) = allow_origin {
            let cors = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_credentials(true)
                .allow_origin(AllowOrigin::predicate(
                    move |origin: &HeaderValue, _request_parts: &request::Parts| {
                        allowed_origin.contains(origin)
                    },
                ));

            self.router = self.router.layer(cors);
        }
        self.with_trace_layer().router
    }

    pub async fn as_standalone(
        self,
        listener: TcpListener,
        allow_origins: Option<AllowedOrigins>,
    ) -> Result<(), std::io::Error> {
        let router = self.as_nested_router(allow_origins);

        tracing::info!("listening on {}", listener.local_addr()?);
        axum_server::Server::<std::net::SocketAddr>::from_listener(listener)
            .serve(router.into_make_service())
            .await
    }

    // pub async fn run(self) -> Result<(), std::io::Error> {
    //     tracing::info!("listening on {}", &self.address);
    //     self.server.await
    // }
    // }
    // pub async fn build(
    //     user_store: U,
    //     banned_token_store: B,
    //     two_fa_code_store: T,
    //     email_client: E,
    //     app_state: AuthServiceState<U, B, T, E>,
    //     address: &str,
    // ) -> Result<Self, Box<dyn Error>> {
    //     let allowed_origins = AuthServiceSetting::load().auth.allowed_origins.clone();

    //     let cors = CorsLayer::new()
    //         .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    //         .allow_credentials(true)
    //         .allow_origin(AllowOrigin::predicate(
    //             move |origin: &HeaderValue, _request_parts: &request::Parts| {
    //                 allowed_origins.contains(origin)
    //             },
    //         ));

    //     let router = Router::new()
    //         .nest_service("/", ServeDir::new("assets"))
    //         .route("/signup", post(signup))
    //         .route("/login", post(login))
    //         .route("/logout", post(logout))
    //         .route("/verify-2fa", post(verify_two_fa))
    //         .route("/verify-token", post(verify_token))
    //         .route("/verify-elevated-token", post(verify_elevated_token))
    //         .route("/elevate", post(elevate))
    //         .route("/change-password", post(change_password))
    //         .route("/delete-account", delete(delete_account))
    //         .with_state(app_state)
    //         .layer(cors)
    //         .layer(
    //             TraceLayer::new_for_http()
    //                 .make_span_with(make_span_with_request_id)
    //                 .on_request(on_request)
    //                 .on_response(on_response),
    //         );

    //     let listener = tokio::net::TcpListener::bind(address).await?;
    //     let address = listener.local_addr()?.to_string();
    //     let server = axum::serve(listener, router);

    //     Ok(AuthService { server, address })
    // }

    // pub async fn run(self) -> Result<(), std::io::Error> {
    //     tracing::info!("listening on {}", &self.address);
    //     self.server.await
    // }
}

pub async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let config = AuthServiceSetting::load();
    let db_url = config.postgres.url.expose_secret();
    let pg_pool = get_postgres_pool(db_url)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

pub fn configure_redis() -> redis::Connection {
    let redis_host_name = &AuthServiceSetting::load().redis.host_name;
    get_redis_client(redis_host_name)
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    // Create a new PostgreSQL connection pool
    PgPoolOptions::new().max_connections(5).connect(url).await
}

pub fn get_redis_client(redis_hostname: &str) -> RedisResult<Client> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}
