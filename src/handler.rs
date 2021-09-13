use crate::{command, session::SmtpState};
use async_trait::async_trait;

/// Handler for SMTP events.
#[async_trait]
pub trait Handler: Send + Sync {
    /// Validate the recipient to be local.
    /// Return false to reject the recipient.
    async fn recipient_local(&self, _recipient: &command::Mailbox) -> bool;
    /// Save an email to the system.
    /// Return true to accept the email.
    async fn save(&self, _state: &SmtpState) -> bool;
}
