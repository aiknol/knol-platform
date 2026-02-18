//! Email channel adapter — lettre SMTP transport for newsletters.

use tracing::{info, warn};

use super::{PublishContent, PublishResult};
use crate::config::ChannelCredentials;
use crate::error::MarketingError;

pub async fn publish(
    content: &PublishContent,
    credentials: &ChannelCredentials,
) -> Result<PublishResult, MarketingError> {
    let smtp_host = credentials.smtp_host.as_ref().ok_or_else(|| {
        MarketingError::Channel {
            channel: "email".into(),
            message: "SMTP credentials not configured".into(),
        }
    })?;
    let smtp_user = credentials.smtp_user.as_deref().unwrap_or("");
    let smtp_pass = credentials.smtp_pass.as_deref().unwrap_or("");
    let from_addr = "noreply@aiknol.com";

    let subject = content
        .subject
        .as_deref()
        .or(content.title.as_deref())
        .unwrap_or("Knol Newsletter");

    let body_html = content.body.as_deref().unwrap_or(&content.text);

    // Build the email message
    let email = lettre::Message::builder()
        .from(from_addr.parse().map_err(|e| MarketingError::Channel {
            channel: "email".into(),
            message: format!("Invalid from address: {}", e),
        })?)
        .to("subscribers@aiknol.com"
            .parse()
            .map_err(|e| MarketingError::Channel {
                channel: "email".into(),
                message: format!("Invalid to address: {}", e),
            })?)
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(body_html.to_string())
        .map_err(|e| MarketingError::Channel {
            channel: "email".into(),
            message: format!("Email build error: {}", e),
        })?;

    // Build SMTP transport
    use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

    let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_host)
        .map_err(|e| MarketingError::Channel {
            channel: "email".into(),
            message: format!("SMTP connection error: {}", e),
        })?
        .credentials(lettre::transport::smtp::authentication::Credentials::new(
            smtp_user.to_string(),
            smtp_pass.to_string(),
        ))
        .build();

    match transport.send(email).await {
        Ok(response) => {
            info!("Email: sent newsletter — {:?}", response);
            Ok(PublishResult::success(
                "email",
                Some(format!("{:?}", response)),
                None,
            ))
        }
        Err(e) => {
            warn!("Email: SMTP error — {}", e);
            Ok(PublishResult::failure("email", format!("SMTP: {}", e)))
        }
    }
}
