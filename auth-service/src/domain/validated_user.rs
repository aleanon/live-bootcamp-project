use super::email::Email;

#[derive(Debug, PartialEq)]
pub enum ValidatedUser {
    Without2Fa(Email),
    With2Fa(Email),
}

impl ValidatedUser {
    pub fn new(email: Email, requires_2fa: bool) -> Self {
        if requires_2fa {
            Self::With2Fa(email)
        } else {
            Self::Without2Fa(email)
        }
    }

    pub fn email(&self) -> &Email {
        match self {
            ValidatedUser::Without2Fa(email) => email,
            ValidatedUser::With2Fa(email) => email,
        }
    }
}
