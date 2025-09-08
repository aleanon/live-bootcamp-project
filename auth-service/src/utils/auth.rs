use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{data_stores::BannedTokenStore, email::Email};

use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};

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
    UnexpectedError,
}

pub fn extract_token<'a>(jar: &'a CookieJar) -> Result<&'a Cookie<'a>, TokenAuthError> {
    match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => Ok(cookie),
        None => Err(TokenAuthError::MissingToken),
    }
}

// Create cookie with a new JWT auth token
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, TokenAuthError> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

// Create cookie and set the value to the passed-in token string
fn create_auth_cookie(token: String) -> Cookie<'static> {
    Cookie::build((JWT_COOKIE_NAME, token))
        .path("/") // apply cookie to all URLs on the server
        .http_only(true) // prevent JavaScript from accessing the cookie
        .secure(true)
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .build()
}

// This value determines how long the JWT auth token is valid for
pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

// Create JWT auth token
fn generate_auth_token(email: &Email) -> Result<String, TokenAuthError> {
    let delta =
        chrono::Duration::try_seconds(TOKEN_TTL_SECONDS).ok_or(TokenAuthError::UnexpectedError)?;

    // Create JWT expiration time
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(TokenAuthError::UnexpectedError)?
        .timestamp();

    // Cast exp to a usize, which is what Claims expects
    let exp: usize = exp
        .try_into()
        .map_err(|_| TokenAuthError::UnexpectedError)?;

    let sub = email.as_ref().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims)
}

// Check if JWT auth token is valid by decoding it using the JWT secret
pub async fn validate_token(
    token: &str,
    banned_token_store: &dyn BannedTokenStore,
) -> Result<Claims, TokenAuthError> {
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(TokenAuthError::TokenError)?;

    let token = create_token(&claims)?;

    let is_banned = banned_token_store
        .token_is_banned(&token)
        .await
        .map_err(|_| TokenAuthError::UnexpectedError)?;

    if is_banned {
        return Err(TokenAuthError::TokenIsBanned);
    }

    Ok(claims)
}

// Create JWT auth token by encoding claims using the JWT secret
fn create_token(claims: &Claims) -> Result<String, TokenAuthError> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
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
    use crate::services::hashset_banned_token_store::HashSetBannedTokenStore;

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
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let banned_token_store = HashSetBannedTokenStore::default();
        let token = generate_auth_token(&email).unwrap();
        let result = validate_token(&token, &banned_token_store).await.unwrap();
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
        let result = validate_token(&token, &banned_token_store).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ban_token() {
        let email = Email::try_from("test@example.com".to_owned()).unwrap();
        let mut banned_token_store = HashSetBannedTokenStore::default();
        let token = generate_auth_token(&email).unwrap();

        banned_token_store.ban_token(token.clone()).await.unwrap();
        let result = validate_token(&token, &banned_token_store).await;
        assert!(result.is_err());
    }
}
