use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use super::notifier::Notifier;
use crate::modules::alerts::application::dto::WebhookPayload;
use crate::modules::alerts::domain::AlertDomainError;

/// Webhook notifier - sends alerts to HTTP endpoints
pub struct WebhookNotifier {
    client: Client,
}

impl WebhookNotifier {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

impl Default for WebhookNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Notifier for WebhookNotifier {
    async fn send(
        &self,
        payload: &WebhookPayload,
        channel_config: &Value,
    ) -> Result<(), AlertDomainError> {
        // Get URL from config
        let url = channel_config
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AlertDomainError::InvalidChannelConfig("Webhook config missing 'url'".to_string())
            })?;

        // Get optional headers from config
        let headers: HashMap<String, String> = channel_config
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        // Get optional secret for signature
        let _secret = channel_config
            .get("secret")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Build request
        let mut request = self.client.post(url).json(&payload);

        // Add custom headers
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        // Send request
        let response = request.send().await.map_err(|e| {
            AlertDomainError::InternalError(format!("Webhook request failed: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());

            tracing::warn!(
                url,
                status = %status,
                body = %body,
                "Webhook returned non-success status"
            );

            return Err(AlertDomainError::InternalError(format!(
                "Webhook returned status {}: {}",
                status, body
            )));
        }

        tracing::debug!(url, "Webhook notification sent successfully");

        Ok(())
    }
}
