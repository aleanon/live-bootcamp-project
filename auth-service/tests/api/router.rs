use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "test/html");
}

#[tokio::test]
async fn signup_returns_200() {
    let app = TestApp::new().await;

    let email = "test@example.com".to_owned();
    let password = "password".to_owned();

    let response = app.sign_up(email, password).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn login_returns_200() {
    let app = TestApp::new().await;

    let email = "test@example.com".to_owned();
    let password = "password".to_owned();

    let response = app.login(email, password).await;

    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn logout_returns_200() {
    let app = TestApp::new().await;

    let email = "test@example.com".to_owned();
    let password = "password".to_owned();

    let response = app.login(email, password).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 200);
}

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

#[tokio::test]
async fn verify_token_returns_200() {
    let app = TestApp::new().await;

    let email = "test@example.com".to_owned();
    let password = "password".to_owned();
    let code = "123456".to_owned();

    let response = app.login(email, password).await;

    assert_eq!(response.status().as_u16(), 200);

    let response = app.verify_2fa(code).await;
    assert_eq!(response.status().as_u16(), 200);

    let token = response.json().await.unwrap();

    let response = app.verify_token(token).await;

    assert_eq!(response.status().as_u16(), 200);
}
