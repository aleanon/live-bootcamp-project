use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    app_state::AppState,
    domain::{auth_api_error::AuthApiError, data_stores::UserStore, user::User},
    requests::signup::SignupRequest,
};

#[tracing::instrument(name = "Signup", skip_all, err(Debug))]
pub async fn signup(
    State(app_state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthApiError> {
    let user = User::parse(request.email, request.password, request.requires_2fa)?;

    app_state.user_store.write().await.add_user(user).await?;

    Ok((
        StatusCode::CREATED,
        String::from("User created successfully!"),
    ))
}
