pub mod app_state;
pub mod domain;
mod routes;
pub mod services;
pub mod utils;

use app_state::AppState;
use axum::{http::Method, routing::post, serve::Serve, Router};
use routes::{delete_account, login, logout, signup, verify_2fa, verify_token};
use std::error::Error;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = [
            "http://127.0.0.1:8000".parse()?,
            "http://134.122.65.215:8000".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .route("/delete-account", post(delete_account))
            .with_state(app_state)
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
