use thiserror::Error;

use super::email::Email;

#[derive(Debug, Error)]
pub enum EmailClientError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] color_eyre::Report),
}

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError>;
}
