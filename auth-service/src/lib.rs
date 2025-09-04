pub mod app_state;
pub mod domain;
mod routes;
pub mod services;
pub mod utils;

use app_state::AppState;
use axum::{routing::post, serve::Serve, Router};
use routes::{delete_account, login, logout, signup, verify_2fa, verify_token};
use std::error::Error;
use tower_http::services::ServeDir;
use utils::dynamic_cors::dynamic_cors;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let config = app_state.config.clone();
        utils::config::listen_for_config_updates(config);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .route("/delete-account", post(delete_account))
            .layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                dynamic_cors,
            ))
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
