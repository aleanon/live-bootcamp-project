use axum::http::HeaderValue;
use dashmap::DashSet;
use dotenvy::dotenv;

use crate::{
    domain::data_stores::AllowedOriginsStore, utils::constants::AUTH_SERVICE_ALLOWED_ORIGINS,
};

#[derive(Debug, Clone)]
pub struct DashSetAllowedOriginsStore {
    allowed_origins: DashSet<HeaderValue>,
}

impl DashSetAllowedOriginsStore {
    pub fn new(allowed_origins: DashSet<HeaderValue>) -> Self {
        Self { allowed_origins }
    }
}

impl Default for DashSetAllowedOriginsStore {
    fn default() -> Self {
        dotenv().ok();

        let allowed_origins = AUTH_SERVICE_ALLOWED_ORIGINS
            .split(',')
            .filter_map(|origin| origin.trim().parse().ok())
            .collect::<DashSet<_>>();

        Self { allowed_origins }
    }
}

impl AllowedOriginsStore for DashSetAllowedOriginsStore {
    fn contains(&self, origin: HeaderValue) -> bool {
        self.allowed_origins.contains(&origin)
    }

    fn add_allowed_origin(
        &self,
        origin: impl IntoIterator<Item = HeaderValue>,
    ) -> Result<(), String> {
        for origin in origin {
            self.allowed_origins.insert(origin);
        }
        Ok(())
    }

    fn remove_allowed_origin(
        &self,
        origin: impl IntoIterator<Item = HeaderValue>,
    ) -> Result<(), String> {
        for origin in origin {
            self.allowed_origins.remove(&origin);
        }
        Ok(())
    }
}
