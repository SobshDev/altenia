use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct AlertRuleRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub config: Value,
    pub threshold_value: f64,
    pub threshold_operator: String,
    pub time_window_seconds: i32,
    pub is_enabled: bool,
    pub last_evaluated_at: Option<DateTime<Utc>>,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, FromRow)]
pub struct AlertChannelRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub channel_type: String,
    pub config: Value,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct AlertRow {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub trigger_value: Option<f64>,
    pub message: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, FromRow)]
pub struct RuleChannelRow {
    pub rule_id: Uuid,
    pub channel_id: Uuid,
}
