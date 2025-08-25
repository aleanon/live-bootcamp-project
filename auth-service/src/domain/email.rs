use std::sync::LazyLock;

use regex::Regex;

use super::user::UserError;

const EMAIL_REGEX_PATTERN: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(EMAIL_REGEX_PATTERN).unwrap());

#[derive(Debug, PartialEq, Clone)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = UserError;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        if !EMAIL_REGEX.is_match(&email) {
            return Err(UserError::InvalidEmail);
        }
        Ok(Email(email))
    }
}

impl Email {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
