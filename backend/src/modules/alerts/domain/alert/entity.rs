use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::modules::alerts::domain::alert_rule::AlertRuleId;
use crate::modules::projects::domain::ProjectId;

/// Alert ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlertId(String);

impl AlertId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Alert Status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertStatus {
    Firing,
    Resolved,
}

impl AlertStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "firing" => Self::Firing,
            "resolved" => Self::Resolved,
            _ => Self::Firing, // Default to firing
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Firing => "firing",
            Self::Resolved => "resolved",
        }
    }
}

impl std::fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Alert - a triggered alert instance
#[derive(Debug, Clone)]
pub struct Alert {
    id: AlertId,
    rule_id: AlertRuleId,
    project_id: ProjectId,
    status: AlertStatus,
    triggered_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    trigger_value: Option<f64>,
    message: Option<String>,
    metadata: Option<Value>,
}

impl Alert {
    pub fn new(
        id: AlertId,
        rule_id: AlertRuleId,
        project_id: ProjectId,
        trigger_value: f64,
        message: String,
        metadata: Option<Value>,
    ) -> Self {
        Self {
            id,
            rule_id,
            project_id,
            status: AlertStatus::Firing,
            triggered_at: Utc::now(),
            resolved_at: None,
            trigger_value: Some(trigger_value),
            message: Some(message),
            metadata,
        }
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_db(
        id: AlertId,
        rule_id: AlertRuleId,
        project_id: ProjectId,
        status: AlertStatus,
        triggered_at: DateTime<Utc>,
        resolved_at: Option<DateTime<Utc>>,
        trigger_value: Option<f64>,
        message: Option<String>,
        metadata: Option<Value>,
    ) -> Self {
        Self {
            id,
            rule_id,
            project_id,
            status,
            triggered_at,
            resolved_at,
            trigger_value,
            message,
            metadata,
        }
    }

    // Getters
    pub fn id(&self) -> &AlertId {
        &self.id
    }

    pub fn rule_id(&self) -> &AlertRuleId {
        &self.rule_id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn status(&self) -> &AlertStatus {
        &self.status
    }

    pub fn triggered_at(&self) -> DateTime<Utc> {
        self.triggered_at
    }

    pub fn resolved_at(&self) -> Option<DateTime<Utc>> {
        self.resolved_at
    }

    pub fn trigger_value(&self) -> Option<f64> {
        self.trigger_value
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn metadata(&self) -> Option<&Value> {
        self.metadata.as_ref()
    }

    pub fn is_firing(&self) -> bool {
        matches!(self.status, AlertStatus::Firing)
    }

    // Mutators
    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Utc::now());
    }
}
