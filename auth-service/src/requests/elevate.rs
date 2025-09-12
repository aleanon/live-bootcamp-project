use serde::Deserialize;

use crate::domain::{email::Email, password::Password, user::UserError};

#[derive(Debug, Deserialize)]
pub struct ElevateRequest {
    pub email: String,
    pub password: String,
}

pub struct ValidElevateRequest {
    email: Email,
    password: Password,
}

impl ValidElevateRequest {
    pub fn parse(elevate_request: ElevateRequest) -> Result<Self, UserError> {
        Ok(ValidElevateRequest {
            email: Email::try_from(elevate_request.email)?,
            password: Password::try_from(elevate_request.password)?,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }
}
