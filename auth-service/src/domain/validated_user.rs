use super::email::Email;

#[derive(Debug, PartialEq)]
pub struct ValidatedUser(Email);

impl ValidatedUser {
    pub fn new(email: Email) -> Self {
        Self(email)
    }

    pub fn email(&self) -> &Email {
        &self.0
    }
}
