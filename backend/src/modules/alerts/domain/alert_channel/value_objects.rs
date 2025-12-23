use crate::modules::alerts::domain::AlertDomainError;

/// Alert Channel ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlertChannelId(String);

impl AlertChannelId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Channel Type - what kind of notification channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelType {
    Webhook,
    // Future: Email, Slack, PagerDuty, etc.
}

impl ChannelType {
    pub fn from_str(s: &str) -> Result<Self, AlertDomainError> {
        match s.to_lowercase().as_str() {
            "webhook" => Ok(Self::Webhook),
            _ => Err(AlertDomainError::InvalidChannelType(format!(
                "Unknown channel type: {}. Valid types: webhook",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Webhook => "webhook",
        }
    }
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
