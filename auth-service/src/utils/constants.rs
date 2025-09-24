use dotenvy::dotenv;
use secrecy::Secret;
use std::{env as std_env, sync::LazyLock};

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
pub static JWT_SECRET: LazyLock<Secret<String>> = LazyLock::new(set_jwt_secret);
pub static JWT_ELEVATED_SECRET: LazyLock<Secret<String>> = LazyLock::new(set_elevated_jwt_secret);
pub static AUTH_SERVICE_ALLOWED_ORIGINS: LazyLock<String> = LazyLock::new(set_allowed_origins);
pub static DATABASE_URL: LazyLock<Secret<String>> = LazyLock::new(set_database_url);
pub static REDIS_HOST_NAME: LazyLock<String> = LazyLock::new(set_redis_host_name);
pub static POSTMARK_AUTH_TOKEN: LazyLock<Secret<String>> = LazyLock::new(set_postmark_auth_token);

fn set_jwt_secret() -> Secret<String> {
    dotenv().ok(); // Load environment variables
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    Secret::new(secret)
}

fn set_elevated_jwt_secret() -> Secret<String> {
    dotenv().ok();
    let secret =
        std_env::var(env::JWT_ELEVATED_SECRET_ENV_VAR).expect("JWT_ELEVATED_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_ELEVATED_SECRET must not be empty.");
    }
    Secret::new(secret)
}

fn set_allowed_origins() -> String {
    dotenv().ok();
    std_env::var(env::AUTH_SERVICE_ALLOWED_ORIGINS_ENV_VAR)
        .unwrap_or("http://127.0.0.1:8000,http://localhost:8000".to_owned())
}

fn set_database_url() -> Secret<String> {
    dotenv().ok();
    Secret::new(std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set."))
}

fn set_redis_host_name() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}

fn set_postmark_auth_token() -> Secret<String> {
    dotenv().ok();
    let token =
        std_env::var(env::POSTMARK_AUTH_TOKEN_ENV_VAR).expect("POSTMARK_AUTH_TOKEN must be set.");
    if token.is_empty() {
        panic!("POSTMARK_AUTH_TOKEN must not be empty.");
    }
    Secret::new(token)
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const JWT_ELEVATED_SECRET_ENV_VAR: &str = "JWT_ELEVATED_SECRET";
    pub const AUTH_SERVICE_ALLOWED_ORIGINS_ENV_VAR: &str = "AUTH_SERVICE_ALLOWED_ORIGINS";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const POSTMARK_AUTH_TOKEN_ENV_VAR: &str = "POSTMARK_AUTH_TOKEN";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const JWT_ELEVATED_COOKIE_NAME: &str = "jwt_elevated";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
    pub mod email_client {
        use std::time::Duration;

        pub const BASE_URL: &str = "https://api.postmarkapp.com/";
        // If you created your own Postmark account, make sure to use your email address!
        pub const SENDER: &str = "bogdan@codeiron.io";
        pub const TIMEOUT: Duration = std::time::Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "test@email.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_millis(200);
    }
}
