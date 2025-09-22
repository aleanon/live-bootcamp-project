use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordVerifier, Version,
    password_hash::{PasswordHasher, SaltString, rand_core},
};
use color_eyre::eyre::{Result, eyre};
use sqlx::{Pool, Postgres};

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::{User, ValidatedUser},
};

pub struct PostgresUserStore {
    pool: sqlx::PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: Pool<Postgres>) -> Self {
        PostgresUserStore { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password = user.password().clone();
        let password_hash = compute_password_hash(password)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e))?;

        let query = sqlx::query!(
            r#"
                INSERT INTO users (email, password_hash, requires_2fa)
                VALUES ($1, $2, $3)
            "#,
            user.email().as_ref(),
            password_hash,
            user.requires_2fa()
        );

        query.execute(&self.pool).await.map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.constraint().is_some() {
                    return UserStoreError::UserAlreadyExists;
                }
            }
            UserStoreError::UnexpectedError(eyre!(e))
        })?;

        Ok(())
    }
    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn authenticate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<ValidatedUser, UserStoreError> {
        let query = sqlx::query!(
            r#"
                SELECT email, password_hash, requires_2fa
                FROM users
                WHERE email = $1
            "#,
            email.as_ref()
        );

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;

        let Some(row) = row else {
            return Err(UserStoreError::UserNotFound);
        };

        let password = password.as_ref().to_owned();

        verify_password_hash(row.password_hash, password)
            .await
            .map_err(|_| UserStoreError::IncorrectPassword)?;

        let email =
            Email::try_from(row.email).map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;
        Ok(ValidatedUser::new(email, row.requires_2fa))
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let query = sqlx::query!(
            r#"
                SELECT email, password_hash, requires_2fa
                FROM users
                WHERE email = $1
            "#,
            email.as_ref()
        );

        let row = query
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;

        let Some(row) = row else {
            return Err(UserStoreError::UserNotFound);
        };

        let user = User::parse(row.email, row.password_hash, row.requires_2fa)
            .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        Ok(user)
    }

    #[tracing::instrument(name = "Delete user from user store", skip_all)]
    async fn delete_user(&mut self, user: &Email) -> Result<(), UserStoreError> {
        let query = sqlx::query!(
            r#"
                DELETE FROM users
                WHERE email = $1
            "#,
            user.as_ref()
        );

        let result = query
            .execute(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }
}

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;

            Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| e.into())
        })
    })
    .await?;

    result
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: Password) -> Result<String> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(move || {
            let salt: SaltString = SaltString::generate(rand_core::OsRng);
            let hasher = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            );
            hasher
                .hash_password(password.as_ref().as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(Into::into)
        })
    })
    .await?;

    result
}

#[cfg(test)]
mod tests {

    use crate::application::get_postgres_pool;

    use super::*;
    use sqlx::PgPool;
    use testcontainers_modules::{
        postgres,
        testcontainers::{ContainerAsync, runners::AsyncRunner},
    };

    async fn setup_and_connect_db_container() -> (ContainerAsync<postgres::Postgres>, PgPool) {
        let container = postgres::Postgres::default()
            .start()
            .await
            .expect("Failed to start container");

        let db_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get the mapped port of the container");

        let host = container
            .get_host()
            .await
            .expect("Failed to get the container host address");

        let db_url = format!("postgres://postgres:postgres@{}:{}", host, db_port);

        let connection = get_postgres_pool(&db_url)
            .await
            .expect("Failed to connect to database");

        sqlx::migrate!()
            .run(&connection)
            .await
            .expect("Failed to migrate the database");

        (container, connection)
    }

    // static POSTGRES_CONTAINER: OnceCell<RwLock<ContainerAsync<postgres::Postgres>>> =
    //     OnceCell::const_new();

    // async fn container() -> RwLockReadGuard<'static, ContainerAsync<postgres::Postgres>> {
    //     POSTGRES_CONTAINER
    //         .get_or_init(async || {
    //             let container = postgres::Postgres::default()
    //                 .start()
    //                 .await
    //                 .expect("Failed to start db container");
    //             RwLock::new(container)
    //         })
    //         .await
    //         .read()
    //         .await
    // }

    // async fn setup_postgres_test_db() -> PgPool {
    //     let container = container().await;

    //     let db_port = container
    //         .get_host_port_ipv4(5432)
    //         .await
    //         .expect("Failed to get the mapped port of the container");

    //     let host = container
    //         .get_host()
    //         .await
    //         .expect("Failed to get the container host address");

    //     let db_url = format!("postgres://postgres:postgres@{}:{}", host, db_port);

    //     configure_postgresql(db_url).await
    // }

    // async fn configure_postgresql(url: String) -> PgPool {
    //     // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    //     let db_name = Uuid::new_v4().to_string();

    //     configure_database(&url, &db_name).await;

    //     let postgresql_conn_url_with_db = format!("{}/{}", url, db_name);

    //     // Create a new connection pool and return it
    //     get_postgres_pool(&postgresql_conn_url_with_db)
    //         .await
    //         .expect("Failed to create Postgres connection pool!")
    // }

    // async fn configure_database(db_conn_string: &str, db_name: &str) {
    //     // Create database connection
    //     let connection = PgPoolOptions::new()
    //         .connect(db_conn_string)
    //         .await
    //         .expect("Failed to create Postgres connection pool.");

    //     // Create a new database
    //     connection
    //         .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
    //         .await
    //         .expect("Failed to create database.");

    //     // Connect to new database
    //     let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    //     let connection = PgPoolOptions::new()
    //         .connect(&db_conn_string)
    //         .await
    //         .expect("Failed to create Postgres connection pool.");

    //     // Run migrations against new database
    //     sqlx::migrate!()
    //         .run(&connection)
    //         .await
    //         .expect("Failed to migrate the database");
    // }

    fn create_test_user() -> User {
        let unique_id = uuid::Uuid::new_v4();
        User::new(
            Email::try_from(format!("test{}@example.com", unique_id)).unwrap(),
            Password::try_from("password123".to_string()).unwrap(),
            false,
        )
    }

    fn create_test_user_with_2fa() -> User {
        let unique_id = uuid::Uuid::new_v4();
        User::new(
            Email::try_from(format!("test2fa{}@example.com", unique_id)).unwrap(),
            Password::try_from("password123".to_string()).unwrap(),
            true,
        )
    }

    #[tokio::test]
    async fn test_add_user_success() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool.clone());
        let user = create_test_user();

        let result = store.add_user(user.clone()).await;
        assert!(result.is_ok());

        // Verify user was added to database
        let stored_user = store.get_user(user.email()).await;

        assert_eq!(stored_user, Ok(user));
    }

    #[tokio::test]
    async fn test_add_user_duplicate_email() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool);
        let user = create_test_user();

        // Add user first time
        store.add_user(user.clone()).await.unwrap();

        // Try to add same user again
        let result = store.add_user(user).await;
        assert_eq!(result, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_authenticate_user_success() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool);
        let user = create_test_user();
        let email = user.email().clone();
        let password = user.password().clone();

        // Add user first
        store.add_user(user).await.unwrap();

        // Authenticate user
        let result = store.authenticate_user(&email, &password).await;
        assert!(result.is_ok());

        let validated_user = result.unwrap();
        assert_eq!(validated_user.email(), &email);
        assert_eq!(validated_user, ValidatedUser::No2Fa(email));
    }

    #[tokio::test]
    async fn test_authenticate_user_with_2fa() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool);
        let user = create_test_user_with_2fa();
        let email = user.email().clone();
        let password = user.password().clone();

        // Add user first
        store.add_user(user).await.unwrap();

        // Authenticate user
        let result = store.authenticate_user(&email, &password).await;
        assert!(result.is_ok());

        let validated_user = result.unwrap();
        assert_eq!(validated_user.email(), &email);
        assert_eq!(validated_user, ValidatedUser::Requires2Fa(email));
    }

    #[tokio::test]
    async fn test_authenticate_user_not_found() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let store = PostgresUserStore::new(pool);
        let email = Email::try_from("nonexistent@example.com".to_string()).unwrap();
        let password = Password::try_from("password123".to_string()).unwrap();

        let result = store.authenticate_user(&email, &password).await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_authenticate_user_wrong_password() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool);
        let user = create_test_user();
        let email = user.email().clone();
        let wrong_password = Password::try_from("wrongpassword".to_string()).unwrap();

        // Add user first
        store.add_user(user).await.unwrap();

        // Try to authenticate with wrong password
        let result = store.authenticate_user(&email, &wrong_password).await;
        assert_eq!(result, Err(UserStoreError::IncorrectPassword));
    }

    #[tokio::test]
    async fn test_get_user_success() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool);
        let user = create_test_user();
        let email = user.email().clone();

        // Add user first
        store.add_user(user.clone()).await.unwrap();

        // Get user
        let result = store.get_user(&email).await;
        assert!(result.is_ok());

        let retrieved_user = result.unwrap();
        assert_eq!(retrieved_user.email(), user.email());
        assert_eq!(retrieved_user.requires_2fa(), user.requires_2fa());
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let store = PostgresUserStore::new(pool);
        let email = Email::try_from("nonexistent@example.com".to_string()).unwrap();

        let result = store.get_user(&email).await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool.clone());
        let user = create_test_user();
        let email = user.email().clone();

        // Add user first
        store.add_user(user.clone()).await.unwrap();

        // Delete user
        let result = store.delete_user(&email).await;
        assert!(result.is_ok());

        // Verify user was deleted
        let result = store.get_user(user.email()).await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let (_container, pool) = setup_and_connect_db_container().await;
        let mut store = PostgresUserStore::new(pool);
        let email = Email::try_from("nonexistent@example.com".to_string()).unwrap();

        let result = store.delete_user(&email).await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_compute_password_hash() {
        let password = Password::try_from("testpassword123".to_owned()).unwrap();
        let hash_result = compute_password_hash(password.clone()).await;

        assert!(hash_result.is_ok());
        let hash = hash_result.unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, password.as_ref()); // Hash should be different from original password
    }

    #[tokio::test]
    async fn test_verify_password_hash_success() {
        let password = Password::try_from("testpassword123".to_owned()).unwrap();
        let hash = compute_password_hash(password.clone()).await.unwrap();

        let result = verify_password_hash(hash, password.as_ref().to_owned()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_password_hash_failure() {
        let password = Password::try_from("testpassword123".to_owned()).unwrap();
        let wrong_password = Password::try_from("wrongpassword".to_owned()).unwrap();
        let hash = compute_password_hash(password).await.unwrap();

        let result = verify_password_hash(hash, wrong_password.as_ref().to_owned()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_password_hash_invalid_hash() {
        let invalid_hash = "invalid_hash_format";
        let password = "testpassword123";

        let result = verify_password_hash(invalid_hash.to_owned(), password.to_owned()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compute_password_hash_deterministic_salt() {
        let password = Password::try_from("testpassword123".to_owned()).unwrap();
        let hash1 = compute_password_hash(password.clone()).await.unwrap();
        let hash2 = compute_password_hash(password.clone()).await.unwrap();

        // Hashes should be different due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify successfully
        assert!(
            verify_password_hash(hash1, password.clone().as_ref().to_owned())
                .await
                .is_ok()
        );
        assert!(
            verify_password_hash(hash2, password.as_ref().to_owned())
                .await
                .is_ok()
        );
    }
}
