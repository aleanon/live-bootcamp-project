use crate::helpers::TestApp;

#[tokio::test]
async fn signup_returns_200() {
    let app = TestApp::new().await;

    let email = "test@example.com".to_owned();
    let password = "password".to_owned();

    let response = app.sign_up(email, password).await;

    assert_eq!(response.status().as_u16(), 200);
}
