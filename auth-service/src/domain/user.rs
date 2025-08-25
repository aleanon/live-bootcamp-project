use thiserror::Error;

use super::{email::Email, password::Password};

#[derive(Debug, Error, PartialEq)]
pub enum UserError {
    #[error("Invalid Email")]
    InvalidEmail,
    #[error("Invalid Password: Must be at least 8 characters")]
    InvalidPassword,
    #[error("Passwords do not match")]
    PasswordsDoNotMatch,
}

#[derive(Debug, Clone)]
pub struct User {
    email: Email,
    password: Password,
    requires_2fa: bool,
}

impl User {
    pub fn parse(email: String, password: String, requires_2fa: bool) -> Result<Self, UserError> {
        Ok(User {
            email: Email::try_from(email)?,
            password: Password::try_from(password)?,
            requires_2fa,
        })
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_matches(&self, password: &Password) -> bool {
        &self.password == password
    }

    pub fn requires_2fa(&self) -> bool {
        self.requires_2fa
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.email == other.email
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_only_matches_email() {
        let user1 = User::parse(
            "test@example.com".to_owned(),
            "passwordpassword123".to_owned(),
            false,
        )
        .unwrap();
        let user2 = User::parse(
            "test@example.com".to_owned(),
            "passwordpassword".to_owned(),
            true,
        )
        .unwrap();
        let user3 = User::parse(
            "test2@example.com".to_owned(),
            "passwordpassword".to_owned(),
            false,
        )
        .unwrap();

        assert_eq!(user1, user2);
        assert_ne!(user1, user3);
    }
}
