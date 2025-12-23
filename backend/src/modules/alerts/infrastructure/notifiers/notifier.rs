use async_trait::async_trait;
use serde_json::Value;

use crate::modules::alerts::application::dto::WebhookPayload;
use crate::modules::alerts::domain::AlertDomainError;

/// Trait for sending notifications through different channels
#[async_trait]
pub trait Notifier: Send + Sync {
    /// Send a notification with the given payload to the configured channel
    async fn send(
        &self,
        payload: &WebhookPayload,
        channel_config: &Value,
    ) -> Result<(), AlertDomainError>;
}
