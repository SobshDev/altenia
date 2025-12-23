use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AlertDomainError {
    #[error("Invalid rule name: {0}")]
    InvalidRuleName(String),

    #[error("Invalid rule type: {0}")]
    InvalidRuleType(String),

    #[error("Invalid threshold operator: {0}")]
    InvalidThresholdOperator(String),

    #[error("Invalid channel name: {0}")]
    InvalidChannelName(String),

    #[error("Invalid channel type: {0}")]
    InvalidChannelType(String),

    #[error("Invalid channel config: {0}")]
    InvalidChannelConfig(String),

    #[error("Invalid webhook URL: {0}")]
    InvalidWebhookUrl(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Project not found")]
    ProjectNotFound,

    #[error("Project deleted")]
    ProjectDeleted,

    #[error("Not a member of the organization")]
    NotOrgMember,

    #[error("Not authorized")]
    NotAuthorized,

    #[error("Alert rule not found")]
    RuleNotFound,

    #[error("Alert channel not found")]
    ChannelNotFound,

    #[error("Alert not found")]
    AlertNotFound,

    #[error("Rule name already exists: {0}")]
    RuleNameExists(String),

    #[error("Channel name already exists: {0}")]
    ChannelNameExists(String),

    #[error("Alert already resolved")]
    AlertAlreadyResolved,

    #[error("Webhook request failed: {0}")]
    WebhookFailed(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
