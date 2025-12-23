use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ==================== Alert Rule DTOs ====================

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub rule_type: String,
    pub config: Value,
    pub threshold_value: f64,
    pub threshold_operator: String,
    #[serde(default = "default_time_window")]
    pub time_window_seconds: i32,
    #[serde(default)]
    pub channel_ids: Vec<String>,
}

fn default_time_window() -> i32 {
    300 // 5 minutes
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAlertRuleRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(default)]
    pub threshold_value: Option<f64>,
    #[serde(default)]
    pub threshold_operator: Option<String>,
    #[serde(default)]
    pub time_window_seconds: Option<i32>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
    #[serde(default)]
    pub channel_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertRuleResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub rule_type: String,
    pub config: Value,
    pub threshold_value: f64,
    pub threshold_operator: String,
    pub time_window_seconds: i32,
    pub is_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_evaluated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub channel_ids: Vec<String>,
}

// ==================== Alert Channel DTOs ====================

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlertChannelRequest {
    pub name: String,
    pub channel_type: String,
    pub config: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAlertChannelRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertChannelResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub channel_type: String,
    pub config: Value,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Alert DTOs ====================

#[derive(Debug, Clone, Serialize)]
pub struct AlertResponse {
    pub id: String,
    pub rule_id: String,
    pub project_id: String,
    pub status: String,
    pub triggered_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertListResponse {
    pub alerts: Vec<AlertResponse>,
    pub total: i64,
    pub has_more: bool,
}

// ==================== Webhook Payload ====================

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub alert_id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub project_id: String,
    pub project_name: String,
    pub status: String,
    pub triggered_at: DateTime<Utc>,
    pub trigger_value: f64,
    pub threshold: f64,
    pub threshold_operator: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}
