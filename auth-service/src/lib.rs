pub mod app_state;
pub mod domain;
mod routes;
pub mod services;
pub mod utils;

use app_state::AppState;
use axum::{http::Method, routing::post, serve::Serve, Router};
use routes::{delete_account, login, logout, signup, verify_2fa, verify_token};
use std::{env, error::Error};
use tokio::{fs::File, io::AsyncReadExt};
use tower_http::{cors::CorsLayer, services::ServeDir};
use utils::{
    config::ConfigInner,
    dynamic_cors::{self, dynamic_cors},
};

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let config = app_state.config.clone();
        utils::config::listen_for_config_updates(config);

        // let allowed_origins = env::var("AUTH_SERVICE_ALLOWED_ORIGIN")
        //     .unwrap_or("http://127.0.0.1:8000".to_owned())
        //     .split(',')
        //     .filter_map(|origin| origin.to_owned().parse().ok())
        //     .collect::<Vec<_>>();

        // let allowed_origins = [
        //     "http://127.0.0.1:8000".parse()?,
        //     "http://134.122.65.215:8000".parse()?,
        // ];

        // let cors = CorsLayer::new()
        //     .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        //     .allow_credentials(true)
        //     .allow_origin(allowed_origins);

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
