use crate::helpers::{get_standard_test_user, TestApp};

#[tokio::test]
async fn verify_2fa_returns_200() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "password",
    });

    let code = "123456".to_owned();

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.verify_2fa(code).await;

    assert_eq!(response.status().as_u16(), 200);
}
