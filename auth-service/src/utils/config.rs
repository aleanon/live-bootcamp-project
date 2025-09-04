use std::{env, fs::File, io::Write, ops::Deref, sync::Arc};

use axum::http::HeaderValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{io::AsyncReadExt, sync::RwLock};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct Config(pub ConfigInner);

impl Config {
    pub fn load() -> Self {
        let inner = match ConfigInner::load() {
            Ok(config) => config,
            Err(_) => {
                let config = ConfigInner::default();
                if let Ok(mut file) = std::fs::File::create("config.json") {
                    let serialized = serde_json::to_string_pretty(&config).unwrap_or(String::new());
                    file.write_all(serialized.as_bytes()).ok();
                };
                config
            }
        };
        Config(inner)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigInner {
    pub allowed_origins: HeaderValues,
}

impl ConfigInner {
    pub fn load() -> Result<Self, ConfigError> {
        let config_file = File::open("config.json")?;
        let config: ConfigInner = serde_json::from_reader(config_file)?;
        Ok(config)
    }
}

#[derive(Debug)]
pub struct HeaderValues(Vec<HeaderValue>);

impl HeaderValues {
    pub fn new(headers: Vec<HeaderValue>) -> Self {
        HeaderValues(headers)
    }
}

impl Default for HeaderValues {
    fn default() -> Self {
        let allowed_origins = env::var("AUTH_SERVICE_ALLOWED_ORIGIN")
            .unwrap_or("http://127.0.0.1:8000".to_owned())
            .split(',')
            .filter_map(|origin| origin.to_owned().parse().ok())
            .collect::<Vec<_>>();

        HeaderValues(allowed_origins)
    }
}

impl Serialize for HeaderValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let headers = self
            .0
            .iter()
            .map(|header_value| header_value.to_str().unwrap_or(""))
            .collect::<Vec<_>>();

        headers.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HeaderValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let headers = Vec::<String>::deserialize(deserializer)?
            .iter()
            .filter_map(|value| value.parse().ok())
            .collect();

        Ok(HeaderValues::new(headers))
    }
}

impl Deref for HeaderValues {
    type Target = Vec<HeaderValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn listen_for_config_updates(config: Arc<RwLock<Config>>) {
    tokio::spawn(async move {
        let mut last_modified = std::time::SystemTime::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let Ok(mut file) = tokio::fs::File::open("config.json").await else {
                continue;
            };

            let Ok(metadata) = file.metadata().await else {
                continue;
            };

            let Ok(modified) = metadata.modified() else {
                continue;
            };

            if modified > last_modified {
                last_modified = modified;
                let mut read_buf = String::new();

                if let Err(_) = file.read_to_string(&mut read_buf).await {
                    continue;
                };

                let Ok(new_config) = serde_json::from_str::<ConfigInner>(&read_buf) else {
                    continue;
                };
                config.write().await.0 = new_config;
            };
        }
    });
}
