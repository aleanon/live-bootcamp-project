use serde::Deserialize;

use crate::domain::{email::Email, password::Password, user::UserError};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ValidLoginRequest {
    email: Email,
    password: Password,
}

impl ValidLoginRequest {
    pub fn parse(email: String, password: String) -> Result<Self, UserError> {
        Ok(Self {
            email: Email::try_from(email)?,
            password: Password::try_from(password)?,
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
