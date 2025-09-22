use secrecy::Secret;
use serde::Deserialize;

use crate::domain::{email::Email, password::Password, user::UserError};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
}

#[derive(Debug)]
pub struct ValidLoginRequest {
    email: Email,
    password: Password,
}

impl ValidLoginRequest {
    pub fn parse(login_request: LoginRequest) -> Result<Self, UserError> {
        Ok(Self {
            email: Email::try_from(login_request.email)?,
            password: Password::try_from(login_request.password)?,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn password_matches(&self, password: &Password) -> bool {
        &self.password == password
    }
}
