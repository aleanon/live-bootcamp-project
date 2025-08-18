use crate::helpers::TestApp;

#[tokio::test]
async fn verify_2fa_returns_200() {
    let app = TestApp::new().await;

    let email = "test@example.com".to_owned();
    let password = "password".to_owned();
    let code = "123456".to_owned();

    let response = app.login(email, password).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.verify_2fa(code).await;

    assert_eq!(response.status().as_u16(), 200);
}
