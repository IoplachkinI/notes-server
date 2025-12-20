use crate::{
    config::Config,
    dto::{SendEmailRequest, SendEmailResponse},
};

use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

pub struct EmailService {
    sender: String,
    smtp_pass: String,
    smtp_relay: String,
    smtp_username: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailServiceError {
    #[error("Invalid email address format: {0}")]
    AddressFormat(#[from] lettre::address::AddressError),

    #[error("Failed to build email message: {0}")]
    MessageBuild(#[from] lettre::error::Error),

    #[error("SMTP transport error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),

    #[error("Failed to connect to SMTP relay: {0}")]
    SmtpRelay(lettre::transport::smtp::Error),
}

impl EmailService {
    pub fn new(config: Config) -> Self {
        EmailService {
            sender: config.sender,
            smtp_pass: config.smtp_pass,
            smtp_relay: config.smtp_relay,
            smtp_username: config.smtp_username,
        }
    }

    pub async fn send_email(
        &self,
        request: SendEmailRequest,
    ) -> Result<SendEmailResponse, EmailServiceError> {
        let email = Message::builder()
            .from(self.sender.clone().parse()?)
            .to(request.to.clone().parse()?)
            .subject(request.subject.clone())
            .body(request.body)?;

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_pass.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.smtp_relay)
            .map_err(EmailServiceError::SmtpRelay)?
            .credentials(creds)
            .build();

        tracing::info!(
            "Sending email to '{}' with subject '{}'",
            request.to,
            request.subject
        );

        mailer.send(email).await?;

        tracing::info!("Message to {} sent successfully", request.to);

        Ok(SendEmailResponse {
            message: format!("Message to {} sent successfully!", request.to),
        })
    }
}
