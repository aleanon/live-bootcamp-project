use std::{ops::Deref, sync::Arc};

use axum::http::HeaderValue;
use dashmap::DashSet;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::utils::constants::AUTH_SERVICE_ALLOWED_ORIGINS;

#[derive(Debug, Deserialize)]
pub struct Postgres {
    database: Secret<String>,
}

pub(crate) struct Settings {}

// #[derive(Clone)]
// pub struct Settings(Arc<RwLock<Config>>);

// impl Settings {
//     pub async fn new(path: String) -> Self {
//         todo!()
//     }

//     pub async fn get_config(&self) -> RwLockReadGuard<Config> {
//         self.0.read().await
//     }

//     pub async fn configure(&self, f: impl FnOnce(&mut Config)) {
//         f(&mut *self.0.write().await)
//     }

//     async fn save_config(&self) {
//         todo!()
//     }
// }

// struct Config {
//     app_address: String,
//     postgres_config: PostgresConfig,
//     allowed_origins: AllowedOrigins,
// }

// #[derive(Serialize, Deserialize)]
// #[allow(unused)]
// struct PostgresConfig {
//     redis_host_name: String,
//     postgres_port: String,
//     postgres_user: String,
//     postgres_password: String,
//     postgres_database_name: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[allow(unused)]
// struct RedisConfig {}

// #[derive(Debug, Clone)]
// pub struct AllowedOrigins(Arc<DashSet<HeaderValue>>);

// impl AllowedOrigins {
//     pub fn new(headers: Arc<DashSet<HeaderValue>>) -> Self {
//         AllowedOrigins(headers)
//     }
// }

// impl Default for AllowedOrigins {
//     fn default() -> Self {
//         dotenv().ok();
//         let allowed_origins = AUTH_SERVICE_ALLOWED_ORIGINS
//             .split(',')
//             .filter_map(|origin| origin.trim().parse().ok())
//             .collect::<DashSet<_>>();

//         AllowedOrigins(Arc::new(allowed_origins))
//     }
// }

// impl Serialize for AllowedOrigins {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let headers = self
//             .0
//             .iter()
//             .filter_map(|header_value| header_value.to_str().and_then(|h| Ok(h.to_owned())).ok())
//             .collect::<DashSet<_>>();

//         headers.serialize(serializer)
//     }
// }

// impl<'de> Deserialize<'de> for AllowedOrigins {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let headers = Vec::<String>::deserialize(deserializer)?
//             .iter()
//             .filter_map(|value| value.parse().ok())
//             .collect();

//         Ok(AllowedOrigins::new(Arc::new(headers)))
//     }
// }

// impl Deref for AllowedOrigins {
//     type Target = Arc<DashSet<HeaderValue>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
