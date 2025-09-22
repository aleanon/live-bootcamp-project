use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub email: Secret<String>,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_factor_code: String,
}
