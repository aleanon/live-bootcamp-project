use auth_service::domain::{
    auth_api_error::{AuthApiError, ErrorResponse},
    data_stores::UserStoreError,
    user::UserError,
};

use crate::helpers::{get_standard_test_user, TestApp};

#[tokio::test]
async fn login_returns_200() {
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
        "password": "password"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_400_whith_invalid_email() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "email": "test@example.c",
        "password": "password"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 400);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Unable to parse Error response")
            .error,
        AuthApiError::InvalidCredentials(UserError::InvalidEmail).to_string()
    )
}

#[tokio::test]
async fn should_return_400_whith_invalid_password() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "pass"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 400);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Unable to parse Error response")
            .error,
        AuthApiError::InvalidCredentials(UserError::InvalidPassword).to_string()
    )
}

#[tokio::test]
async fn should_return_401_with_wrong_password() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "wrongpassword",
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Unable to parse error response")
            .error,
        AuthApiError::AuthenticationError(Box::new(UserStoreError::IncorrectPassword)).to_string()
    )
}

#[tokio::test]
async fn should_return_401_with_unregistered_email() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "email": "unregistered@example.com",
        "password": "password",
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Unable to parse error response")
            .error,
        AuthApiError::UserNotFound.to_string()
    )
}

#[tokio::test]
async fn should_return_422_with_malformed_input() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "emal": "test@example.com",
        "password": "password"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}
