use std::fmt::Debug;

use super::user::UserError;

#[derive(Clone, PartialEq)]
pub struct Password(String);

impl TryFrom<String> for Password {
    type Error = UserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            Err(UserError::InvalidPassword)
        } else {
            Ok(Password(value))
        }
    }
}

impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password(*Masked*)")
    }
}
