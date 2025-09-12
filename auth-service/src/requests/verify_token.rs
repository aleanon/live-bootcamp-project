use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
