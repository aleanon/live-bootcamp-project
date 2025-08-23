use std::collections::HashMap;

use crate::domain::user::User;

#[derive(Debug, PartialEq, Eq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    UnexpectedError,
    InvalidCredentials,
}

#[derive(Debug, Default)]
pub struct HashMapUserStore {
    users: HashMap<String, User>,
}

impl HashMapUserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email)?;
        if !user.password_matches(password) {
            Err(UserStoreError::InvalidCredentials)
        } else {
            Ok(())
        }
    }

    pub fn create_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(user.email()) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email().to_owned(), user);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_user_succeeds() {
        let mut store = HashMapUserStore::new();
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        assert!(store.create_user(user).is_ok());
    }

    #[test]
    fn test_add_user_fails_if_user_exists() {
        let mut store = HashMapUserStore::new();
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        let user2 = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        assert!(store.create_user(user).is_ok());
        assert_eq!(
            store.create_user(user2),
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[test]
    fn test_get_user() {
        let mut user_store = HashMapUserStore::new();
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        assert!(user_store.create_user(user.clone()).is_ok());
        let user_two = user_store.get_user(user.email()).unwrap();
        assert_eq!(user_two, &user);
    }

    #[test]
    fn test_validate_user() {
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        let mut user_store = HashMapUserStore::new();
        assert!(user_store.create_user(user.clone()).is_ok());
        assert!(user_store
            .validate_user(user.email(), "passwordpassword")
            .is_ok());
        assert_eq!(
            user_store.validate_user(user.email(), "wrongpassword"),
            Err(UserStoreError::InvalidCredentials)
        );
    }
}
