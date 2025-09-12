use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, user::User},
    requests::signup::SignupRequest,
};

pub async fn signup(
    State(app_state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let user = User::parse(request.email, request.password, request.requires_2fa)?;

    let mut user_store = app_state.user_store.write().await;

    if let Err(_) = user_store.add_user(user).await {
        return Err(AuthApiError::UserAlreadyExists);
    }

    Ok((
        StatusCode::CREATED,
        String::from("User created successfully!"),
    ))
}
