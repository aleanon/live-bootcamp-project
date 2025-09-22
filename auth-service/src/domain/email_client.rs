use color_eyre::eyre::Result;

use super::email::Email;

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()>;
}
