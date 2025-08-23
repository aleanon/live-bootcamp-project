use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::{app_state::AppState, domain::user::User};

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    #[serde(rename = "requires2FA")]
    requires_2fa: bool,
}
pub async fn signup(
    State(app_state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let user = match User::parse(request.email, request.password, request.requires_2fa) {
        Ok(credentials) => credentials,
        Err(response) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(response)
                .unwrap()
        }
    };

    let mut user_store = app_state.user_store.write().await;

    if let Err(_) = user_store.create_user(user) {
        return Response::builder()
            .status(StatusCode::CONFLICT)
            .body("User already exists".to_owned())
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::CREATED)
        .body("Account created successfully".to_owned())
        .unwrap()
}
