use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::Utc;
use color_eyre::eyre::eyre;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{data_stores::BannedTokenStore, email::Email};

use super::constants::{
    JWT_COOKIE_NAME, JWT_ELEVATED_COOKIE_NAME, JWT_ELEVATED_SECRET, JWT_SECRET,
};

#[derive(Debug, Error)]
pub enum TokenAuthError {
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token error: {0}")]
    TokenError(jsonwebtoken::errors::Error),
    #[error("Token is banned")]
    TokenIsBanned,
    #[error("Unexpected error")]
    UnexpectedError(#[source] color_eyre::Report),
}

pub fn extract_token<'a>(jar: &'a CookieJar, cookie_name: &str) -> Result<&'a str, TokenAuthError> {
    match jar.get(cookie_name) {
        Some(cookie) => Ok(cookie.value()),
        None => Err(TokenAuthError::MissingToken),
    }
}

// Create cookie with a new JWT auth token
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, TokenAuthError> {
    let token = generate_auth_token(email, TOKEN_TTL_SECONDS, JWT_SECRET.as_bytes())?;
    Ok(create_auth_cookie(token, JWT_COOKIE_NAME))
}

pub fn generate_elevated_auth_cookie(email: &Email) -> Result<Cookie<'static>, TokenAuthError> {
    let token = generate_auth_token(
        email,
        ELEVATED_TOKEN_TTL_SECONDS,
        JWT_ELEVATED_SECRET.as_bytes(),
    )?;
    Ok(create_auth_cookie(token, JWT_ELEVATED_COOKIE_NAME))
}

pub fn create_removal_cookie(cookie_name: &'static str) -> Cookie<'static> {
    let mut cookie = create_auth_cookie(String::new(), cookie_name);
    cookie.make_removal();
    cookie
}

// Create cookie and set the value to the passed-in token string
pub fn create_auth_cookie(token: String, cookie_name: &'static str) -> Cookie<'static> {
    Cookie::build((cookie_name, token))
        .path("/") // apply cookie to all URLs on the server
        .http_only(true) // prevent JavaScript from accessing the cookie
        .secure(true)
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .build()
}

// This value determines how long the JWT auth token is valid for
pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes
pub const ELEVATED_TOKEN_TTL_SECONDS: i64 = 60; // 1 minute

// Create JWT auth token
fn generate_auth_token(
    email: &Email,
    token_ttl_seconds: i64,
    secret: &[u8],
) -> Result<String, TokenAuthError> {
    let delta = chrono::Duration::try_seconds(token_ttl_seconds).ok_or(
        TokenAuthError::UnexpectedError(eyre!("Failed to create auth token duration")),
    )?;

    // Create JWT expiration time
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(TokenAuthError::UnexpectedError(eyre!(
            "Duration out of range"
        )))?
        .timestamp();

    // Cast exp to a usize, which is what Claims expects
    let exp: usize = exp
        .try_into()
        .map_err(|_| TokenAuthError::UnexpectedError(eyre!("Failed to cast i64 to usize")))?;

    let sub = email.as_ref().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims, secret)
}

// Check if JWT auth token is valid by decoding it using the JWT secret
pub async fn validate_auth_token(
    token: &str,
    banned_token_store: &dyn BannedTokenStore,
) -> Result<Claims, TokenAuthError> {
    validate_token(token, banned_token_store, JWT_SECRET.as_bytes()).await
}

pub async fn validate_elevated_auth_token(
    token: &str,
    banned_token_store: &dyn BannedTokenStore,
) -> Result<Claims, TokenAuthError> {
    validate_token(token, banned_token_store, JWT_ELEVATED_SECRET.as_bytes()).await
}

async fn validate_token(
    token: &str,
    banned_token_store: &dyn BannedTokenStore,
    secret: &[u8],
) -> Result<Claims, TokenAuthError> {
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(TokenAuthError::TokenError)?;

    let token = create_token(&claims, secret)?;

    let is_banned = banned_token_store
        .contains_token(&token)
        .await
        .map_err(|e| TokenAuthError::UnexpectedError(eyre!(e)))?;

    if is_banned {
        return Err(TokenAuthError::TokenIsBanned);
    }

    Ok(claims)
}

// Create JWT auth token by encoding claims using the JWT secret
fn create_token(claims: &Claims, secret: &[u8]) -> Result<String, TokenAuthError> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(TokenAuthError::TokenError)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use crate::services::data_stores::hashset_banned_token_store::HashSetBannedTokenStore;

    use super::*;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone(), JWT_COOKIE_NAME);
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let result = generate_auth_token(&email, TOKEN_TTL_SECONDS, JWT_SECRET.as_bytes()).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let banned_token_store = HashSetBannedTokenStore::default();
        let token = generate_auth_token(&email, TOKEN_TTL_SECONDS, JWT_SECRET.as_bytes()).unwrap();
        let result = validate_auth_token(&token, &banned_token_store)
            .await
            .unwrap();
        assert_eq!(result.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let banned_token_store = HashSetBannedTokenStore::default();
        let result = validate_auth_token(&token, &banned_token_store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ban_token() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let mut banned_token_store = HashSetBannedTokenStore::default();
        let token = generate_auth_token(&email, TOKEN_TTL_SECONDS, JWT_SECRET.as_bytes()).unwrap();

        banned_token_store.ban_token(token.clone()).await.unwrap();
        let result = validate_auth_token(&token, &banned_token_store).await;
        assert!(result.is_err());
    }
}
