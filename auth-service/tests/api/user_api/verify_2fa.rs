use auth_service::{
    domain::auth_api_error::{AuthApiError, ErrorResponse},
    routes::Verify2FARequest,
    utils::auth,
};

use crate::helpers::{TestApp, get_standard_test_user};

#[tokio::test]
async fn should_return_200() {
    let app = TestApp::new().await;

    let body = get_standard_test_user(true);
    assert!(app.post_signup(&body).await.status().is_success());

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let Verify2FARequest {
        email,
        login_attempt_id,
        two_factor_code,
    } = app.get_verify_two_fa_request(&body).await;

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_factor_code
    });

    let response = app.verify_2fa(&body).await;
    let token = app.get_jwt_token();
    let banned_token_store = app.banned_token_store.read().await;
    auth::validate_token(&token, &*banned_token_store)
        .await
        .expect("Invalid auth token");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_400_with_invalid_input() {
    let app = TestApp::new().await;

    let body = get_standard_test_user(true);

    assert!(app.post_signup(&body).await.status().is_success());
    assert!(app.login(&body).await.status().as_u16() == 206);

    let Verify2FARequest {
        email,
        mut login_attempt_id,
        two_factor_code,
    } = app.get_verify_two_fa_request(&body).await;

    login_attempt_id.push_str("invalid");

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_factor_code
    });

    let response = app.verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_with_outdated_login_attempt_id() {
    let app = TestApp::new().await;

    let body = get_standard_test_user(true);

    assert!(app.post_signup(&body).await.status().is_success());
    assert!(app.login(&body).await.status().as_u16() == 206);

    let Verify2FARequest {
        email,
        login_attempt_id,
        two_factor_code,
    } = app.get_verify_two_fa_request(&body).await;

    assert!(app.login(&body).await.status().as_u16() == 206);

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_factor_code
    });

    let response = app.verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Failed to parse error response")
            .error,
        AuthApiError::InvalidLoginAttemptId.to_string()
    )
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;

    let body = get_standard_test_user(true);
    assert!(app.post_signup(&body).await.status().is_success());

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let Verify2FARequest {
        email,
        login_attempt_id,
        two_factor_code,
    } = app.get_verify_two_fa_request(&body).await;

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_factor_code
    });

    assert_eq!(app.verify_2fa(&body).await.status().as_u16(), 200);

    assert_eq!(app.verify_2fa(&body).await.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_422_when_malformed_input() {
    let app = TestApp::new().await;

    let body = get_standard_test_user(true);
    assert!(app.post_signup(&body).await.status().is_success());

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 206);

    let Verify2FARequest {
        email,
        login_attempt_id,
        two_factor_code,
    } = app.get_verify_two_fa_request(&body).await;

    let body = serde_json::json!({
        "email": email,
        "loginAttemptI": login_attempt_id, //Missing the last d
        "2FACode": two_factor_code
    });

    let response = app.verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 422);
}
