use std::sync::LazyLock;

use regex::Regex;

const EMAIL_REGEX_PATTERN: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(EMAIL_REGEX_PATTERN).unwrap());

#[derive(Debug, Clone)]
pub struct User {
    email: String,
    password: String,
    requires_2fa: bool,
}

impl User {
    pub fn parse(email: String, password: String, requires_2fa: bool) -> Result<Self, String> {
        if !EMAIL_REGEX.is_match(&email) {
            return Err("Invalid Email".to_owned());
        }
        if password.len() < 8 {
            return Err("Invalid Password: Must be at least 8 characters".to_owned());
        }

        Ok(User {
            email,
            password,
            requires_2fa,
        })
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_matches(&self, password: &str) -> bool {
        self.password == password
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
