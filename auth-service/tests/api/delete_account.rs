use auth_service::domain::{
    auth_api_error::{AuthApiError, ErrorResponse},
    data_stores::UserStoreError,
};

use crate::helpers::{get_standard_test_user, TestApp};

#[tokio::test]
pub async fn delete_account_should_succeed_with_valid_credentials() {
    let client = TestApp::new().await;

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "password",
        "requires2FA": false,
    });

    let user_created = client.post_signup(&body).await;
    assert_eq!(user_created.status().as_u16(), 201);

    let user_deleted = client.delete_account(&body).await;
    let status_code = user_deleted.status().as_u16();
    let reponse_text = user_deleted.text().await.unwrap_or(String::new());
    println!("{}", reponse_text);
    assert_eq!(status_code, 200);
}

#[tokio::test]
pub async fn delete_account_should_return_400_with_incorrect_password() {
    let app = TestApp::new().await;

    assert!(
        app.post_signup(&get_standard_test_user(false))
            .await
            .status()
            .as_u16()
            == 201
    );

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "incorrect_password",
        "requires2FA": false,
    });

    let user_deleted = app.delete_account(&body).await;
    assert_eq!(user_deleted.status().as_u16(), 401);
    assert_eq!(
        user_deleted
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        AuthApiError::AuthenticationError(Box::new(UserStoreError::IncorrectPassword)).to_string()
    );
}

#[tokio::test]
pub async fn delete_account_should_return_404_with_nonexistent_user() {
    let client = TestApp::new().await;

    let body = serde_json::json!({
        "email": "nonexistent@example.com",
        "password": "password",
        "requires2FA": false,
    });

    let response = client.delete_account(&body).await;
    assert_eq!(response.status().as_u16(), 404);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        AuthApiError::UserNotFound.to_string()
    );
}

#[tokio::test]
pub async fn delete_account_should_return_422_with_misformed_input() {
    let client = TestApp::new().await;

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "password",
        "requires2FA": false,
    });

    let response = client.post_signup(&body).await;
    assert_eq!(response.status().as_u16(), 201);

    let body = serde_json::json!({
        "emal": "test@example.com",
        "password": "password",
        "requires2FA": false,
    });

    let response = client.delete_account(&body).await;
    assert_eq!(response.status().as_u16(), 422);
}
