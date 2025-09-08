use crate::domain::{
    email::Email,
    email_client::{EmailClient, EmailClientError},
};

pub struct MockEmailClient;

impl Default for MockEmailClient {
    fn default() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError> {
        println!(
            "recipient: {:?}\nsubject: {}\ncontent: {}",
            recipient, subject, content
        );
        Ok(())
    }
}
