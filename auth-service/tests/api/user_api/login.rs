use auth_service::{
    domain::{
        auth_api_error::{AuthApiError, ErrorResponse},
        data_stores::UserStoreError,
        email::Email,
        two_fa_attempt_id::TwoFaAttemptId,
        user::UserError,
    },
    responses::login::TwoFactorAuthResponse,
};
use secrecy::Secret;

use crate::helpers::{TestApp, get_standard_test_user};

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
async fn should_return_206_when_2fa_enabled() {
    let app = TestApp::new().await;

    assert!(
        app.post_signup(&get_standard_test_user(true))
            .await
            .status()
            .is_success()
    );

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "password"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let response = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Failed to parse response");

    assert_eq!(&response.message, "2FA required");

    let login_id = TwoFaAttemptId::parse(&response.attempt_id).expect("Invalid code");

    let two_fa_code_store = app.two_fa_code_store.read().await;
    let email = Email::try_from(Secret::new("test@example.com".to_string())).unwrap();
    let (login_attempt_id, _) = two_fa_code_store
        .get_login_attempt_id_and_two_fa_code(&email)
        .await
        .unwrap();
    assert_eq!(login_attempt_id, login_id)
}

#[tokio::test]
async fn should_return_400_whith_invalid_email() {
    let app = TestApp::new().await;

    assert!(
        app.post_signup(&get_standard_test_user(false))
            .await
            .status()
            .is_success()
    );

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
        AuthApiError::InvalidInput(Box::new(UserError::InvalidEmail)).to_string()
    )
}

#[tokio::test]
async fn should_return_400_whith_invalid_password() {
    let app = TestApp::new().await;

    assert!(
        app.post_signup(&get_standard_test_user(false))
            .await
            .status()
            .is_success()
    );

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
        AuthApiError::InvalidInput(Box::new(UserError::InvalidPassword)).to_string()
    )
}

#[tokio::test]
async fn should_return_401_with_wrong_password() {
    let app = TestApp::new().await;

    assert!(
        app.post_signup(&get_standard_test_user(false))
            .await
            .status()
            .is_success()
    );

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

    assert!(
        app.post_signup(&get_standard_test_user(false))
            .await
            .status()
            .is_success()
    );

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

    assert!(
        app.post_signup(&get_standard_test_user(false))
            .await
            .status()
            .is_success()
    );

    let body = serde_json::json!({
        "emal": "test@example.com",
        "password": "password"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}
