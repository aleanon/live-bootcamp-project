use serde::Serialize;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn signup_returns_200() {
    let app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "test@example.com",
        "password": "password",
        "requires2FA": false,
    });

    let response = app.post_signup(&body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn signup_returns_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = vec![
        serde_json::json!({
            "password": "password123",
            "requires2FA": false,
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
            "password": 21321989,
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": false,
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": 10
        }),
        serde_json::json!({
            "email": 19299190,
            "password": "password123",
            "requires2FA": true
        }),
    ];

    for body in test_cases {
        let response = app.post_signup(&body).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}
