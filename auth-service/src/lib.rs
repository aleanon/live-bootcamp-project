pub mod admin_routes;
pub mod app_state;
pub mod domain;
pub mod requests;
pub mod responses;
pub mod routes;
pub mod services;
pub mod utils;

use app_state::AppState;
use axum::{
    Router,
    http::{HeaderValue, Method, request},
    routing::{delete, post},
    serve::Serve,
};
use routes::{delete_account, elevate, login, logout, signup, verify_token, verify_two_fa};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::error::Error;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
};

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let config = app_state.config.clone();
        let allowed_origins = config.read().await.allowed_origins.clone();
        utils::config::listen_for_config_updates(config);

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_credentials(true)
            .allow_origin(AllowOrigin::predicate(
                move |origin: &HeaderValue, _request_parts: &request::Parts| {
                    allowed_origins.contains(origin)
                },
            ));

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_two_fa))
            .route("/verify-token", post(verify_token))
            .route("/elevate", post(elevate))
            .route("/delete-account", delete(delete_account))
            .layer(cors)
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    // Create a new PostgreSQL connection pool
    PgPoolOptions::new().max_connections(5).connect(url).await
}
