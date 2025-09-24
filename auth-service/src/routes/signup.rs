use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{
        auth_api_error::AuthApiError,
        data_stores::{BannedTokenStore, TwoFaCodeStore, UserStore},
        email_client::EmailClient,
        user::User,
    },
};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[tracing::instrument(name = "Signup", skip_all, err(Debug))]
pub async fn signup<U, B, T, E>(
    State(app_state): State<AppState<U, B, T, E>>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthApiError>
where
    U: UserStore,
    B: BannedTokenStore,
    T: TwoFaCodeStore,
    E: EmailClient,
{
    let user = User::parse(request.email, request.password, request.requires_2fa)?;

    app_state.user_store.write().await.add_user(user).await?;

    Ok((
        StatusCode::CREATED,
        String::from("User created successfully!"),
    ))
}
