use super::user::UserError;

#[derive(Debug, Clone, PartialEq)]
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
