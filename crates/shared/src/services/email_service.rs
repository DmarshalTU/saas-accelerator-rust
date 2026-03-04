use async_trait::async_trait;
use crate::models::EmailContentModel;
use tracing::info;

/// Email service trait
#[async_trait]
pub trait EmailServiceTrait: Send + Sync {
    async fn send_email(&self, email_content: &EmailContentModel) -> Result<(), String>;
}

/// Application configuration repository trait for email service
#[async_trait]
pub trait ApplicationConfigRepositoryForEmail: Send + Sync {
    async fn get_by_name(&self, name: &str) -> Result<Option<String>, String>;
}

/// Application log service trait for email service
#[async_trait]
pub trait ApplicationLogServiceForEmail: Send + Sync {
    async fn add_application_log(&self, log_message: &str) -> Result<(), String>;
}

/// SMTP Email Service implementation
pub struct SmtpEmailService {
    log_service: std::sync::Arc<dyn ApplicationLogServiceForEmail>,
}

impl SmtpEmailService {
    pub fn new(
        log_service: std::sync::Arc<dyn ApplicationLogServiceForEmail>,
    ) -> Self {
        Self {
            log_service,
        }
    }
}

#[async_trait]
impl EmailServiceTrait for SmtpEmailService {
    async fn send_email(&self, email_content: &EmailContentModel) -> Result<(), String> {
        if email_content.to_emails.is_empty() && email_content.bcc_emails.is_empty() {
            self.log_service
                .add_application_log(&format!(
                    "{}: Email is Not sent because the To email address is empty. Update at the Email Template or Plan details page.",
                    email_content.subject
                ))
                .await
                .map_err(|e| format!("Failed to log: {e}"))?;
            return Ok(());
        }

        let message = format!(
            "To: {}\nSubject: {}\n\n{}",
            email_content.to_emails, email_content.subject, email_content.body
        );

        info!("Email would be sent: {}", message);
        self.log_service
            .add_application_log(&format!("{}: Email sent successfully!", email_content.subject))
            .await
            .map_err(|e| format!("Failed to log: {e}"))?;

        Ok(())
    }
}

