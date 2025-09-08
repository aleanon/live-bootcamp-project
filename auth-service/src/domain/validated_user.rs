use super::email::Email;

#[derive(Debug, PartialEq)]
pub enum ValidatedUser {
    Requires2Fa(Email),
    No2Fa(Email),
}

impl ValidatedUser {
    pub fn new(email: Email, requires_2fa: bool) -> Self {
        if requires_2fa {
            Self::Requires2Fa(email)
        } else {
            Self::No2Fa(email)
        }
    }

    pub fn email(&self) -> &Email {
        match self {
            Self::Requires2Fa(email) => email,
            Self::No2Fa(email) => email,
        }
    }
}
