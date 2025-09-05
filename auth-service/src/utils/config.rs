use std::{fs::File, io::Write, ops::Deref, sync::Arc};

use axum::http::HeaderValue;
use dashmap::DashSet;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{io::AsyncReadExt, sync::RwLock};

use super::constants::AUTH_SERVICE_ALLOWED_ORIGINS;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Config(pub ConfigInner);

impl Config {
    pub fn load() -> Self {
        let inner = match ConfigInner::load() {
            Ok(config_inner) => config_inner,
            Err(_) => {
                let config_inner = ConfigInner::default();
                if let Ok(mut file) = std::fs::File::create("config.json") {
                    let serialized =
                        serde_json::to_string_pretty(&config_inner).unwrap_or(String::new());
                    file.write_all(serialized.as_bytes()).ok();
                };
                config_inner
            }
        };
        Config(inner)
    }
}

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct ConfigInner {
    pub allowed_origins: HeaderValues,
}

impl ConfigInner {
    pub fn load() -> Result<Self, ConfigError> {
        let config_file = File::open("config.json")?;
        let config: ConfigInner = serde_json::from_reader(config_file)?;
        Ok(config)
    }

    pub fn update(&mut self, new_config: ConfigInner) {
        self.allowed_origins
            .retain(|v| new_config.allowed_origins.contains(v));
        for origin in Arc::unwrap_or_clone(new_config.allowed_origins.0) {
            self.allowed_origins.insert(origin);
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeaderValues(Arc<DashSet<HeaderValue>>);

impl HeaderValues {
    pub fn new(headers: Arc<DashSet<HeaderValue>>) -> Self {
        HeaderValues(headers)
    }
}

impl Default for HeaderValues {
    fn default() -> Self {
        dotenv().ok();
        let allowed_origins = AUTH_SERVICE_ALLOWED_ORIGINS
            .split(',')
            .filter_map(|origin| origin.trim().parse().ok())
            .collect::<DashSet<_>>();

        HeaderValues(Arc::new(allowed_origins))
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
            .filter_map(|header_value| header_value.to_str().and_then(|h| Ok(h.to_owned())).ok())
            .collect::<DashSet<_>>();

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

        Ok(HeaderValues::new(Arc::new(headers)))
    }
}

impl Deref for HeaderValues {
    type Target = Arc<DashSet<HeaderValue>>;

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
                config.write().await.0.update(new_config);
            };
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_values_deserialize() {
        let json = r#"["http://example.com", "https://example.org"]"#;
        let result = serde_json::from_str::<HeaderValues>(json);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&HeaderValue::from_static("http://example.com")));
        assert!(values.contains(&HeaderValue::from_static("https://example.org")));
    }
}
