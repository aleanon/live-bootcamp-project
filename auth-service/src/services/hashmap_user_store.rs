use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::User,
    validated_user::ValidatedUser,
};

#[derive(Debug, Default)]
pub struct HashMapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashMapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(user.email()) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        self.users.insert(user.email().to_owned(), user);
        Ok(())
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<ValidatedUser, UserStoreError> {
        let user = self.get_user(email).await?;
        if !user.password_matches(password) {
            Err(UserStoreError::IncorrectPassword)
        } else {
            Ok(ValidatedUser::new(email.clone(), user.requires_2fa()))
        }
    }

    async fn get_user(&self, email: &Email) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn delete_user(&mut self, user: &ValidatedUser) -> Result<(), UserStoreError> {
        self.users
            .remove(user.email())
            .map(|_| ())
            .ok_or(UserStoreError::UserNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user_succeeds() {
        let mut store = HashMapUserStore::default();
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        assert!(store.add_user(user).await.is_ok());
    }

    #[tokio::test]
    async fn test_add_user_fails_if_user_exists() {
        let mut store = HashMapUserStore::default();
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
        assert!(store.add_user(user).await.is_ok());
        assert_eq!(
            store.add_user(user2).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashMapUserStore::default();
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        assert!(user_store.add_user(user.clone()).await.is_ok());
        let user_two = user_store.get_user(user.email()).await.unwrap();
        assert_eq!(user_two, &user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        let mut user_store = HashMapUserStore::default();
        assert!(user_store.add_user(user.clone()).await.is_ok());
        assert!(user_store
            .validate_user(
                user.email(),
                &Password::try_from("passwordpassword".to_string()).unwrap()
            )
            .await
            .is_ok());
        assert_eq!(
            user_store
                .validate_user(
                    user.email(),
                    &Password::try_from("wrongpassword".to_string()).unwrap()
                )
                .await,
            Err(UserStoreError::IncorrectPassword)
        );
    }

    #[tokio::test]
    async fn test_delete_user() {
        let user = User::parse(
            "test@example.com".to_string(),
            "passwordpassword".to_string(),
            false,
        )
        .unwrap();
        let mut user_store = HashMapUserStore::default();
        assert!(user_store.add_user(user.clone()).await.is_ok());
        let validated_user = user_store
            .validate_user(user.email(), user.password())
            .await
            .expect("Unable to validate user");
        assert!(user_store.delete_user(&validated_user).await.is_ok());
        assert!(user_store.get_user(user.email()).await.is_err());
    }
}
