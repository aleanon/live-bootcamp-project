use crate::helpers::{get_standard_test_user, TestApp};

#[tokio::test]
async fn logout_returns_200() {
    let app = TestApp::new().await;

    assert!(app
        .post_signup(&get_standard_test_user(false))
        .await
        .status()
        .is_success());

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "password"
    });

    let response = app.login(&body).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);
}
