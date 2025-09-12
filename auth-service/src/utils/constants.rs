use dotenvy::dotenv;
use std::{env as std_env, sync::LazyLock};

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
pub static JWT_SECRET: LazyLock<String> = LazyLock::new(|| set_token());
pub static JWT_ELEVATED_SECRET: LazyLock<String> = LazyLock::new(|| set_elevated_token());
pub static AUTH_SERVICE_ALLOWED_ORIGINS: LazyLock<String> = LazyLock::new(|| set_allowed_origins());

fn set_token() -> String {
    dotenv().ok(); // Load environment variables
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    secret
}

fn set_elevated_token() -> String {
    dotenv().ok();
    let secret =
        std_env::var(env::JWT_ELEVATED_SECRET_ENV_VAR).expect("JWT_ELEVATED_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_ELEVATED_SECRET must not be empty.");
    }
    secret
}

fn set_allowed_origins() -> String {
    dotenv().ok();
    std_env::var(env::AUTH_SERVICE_ALLOWED_ORIGINS_ENV_VAR)
        .unwrap_or("http://127.0.0.1:8000,http://localhost:8000".to_owned())
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const JWT_ELEVATED_SECRET_ENV_VAR: &str = "JWT_ELEVATED_SECRET";
    pub const AUTH_SERVICE_ALLOWED_ORIGINS_ENV_VAR: &str = "AUTH_SERVICE_ALLOWED_ORIGINS";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const JWT_ELEVATED_COOKIE_NAME: &str = "jwt_elevated";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
