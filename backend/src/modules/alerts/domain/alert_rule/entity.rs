use chrono::{DateTime, Utc};
use serde_json::Value;

use super::value_objects::{AlertRuleId, RuleType, ThresholdOperator};
use crate::modules::auth::domain::UserId;
use crate::modules::projects::domain::ProjectId;

/// Alert Rule - defines when an alert should trigger
#[derive(Debug, Clone)]
pub struct AlertRule {
    id: AlertRuleId,
    project_id: ProjectId,
    name: String,
    description: Option<String>,
    rule_type: RuleType,
    config: Value,
    threshold_value: f64,
    threshold_operator: ThresholdOperator,
    time_window_seconds: i32,
    is_enabled: bool,
    last_evaluated_at: Option<DateTime<Utc>>,
    last_triggered_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: UserId,
    /// Channel IDs associated with this rule
    channel_ids: Vec<String>,
}

impl AlertRule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: AlertRuleId,
        project_id: ProjectId,
        name: String,
        description: Option<String>,
        rule_type: RuleType,
        config: Value,
        threshold_value: f64,
        threshold_operator: ThresholdOperator,
        time_window_seconds: i32,
        created_by: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            project_id,
            name,
            description,
            rule_type,
            config,
            threshold_value,
            threshold_operator,
            time_window_seconds,
            is_enabled: true,
            last_evaluated_at: None,
            last_triggered_at: None,
            created_at: now,
            updated_at: now,
            created_by,
            channel_ids: Vec::new(),
        }
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_db(
        id: AlertRuleId,
        project_id: ProjectId,
        name: String,
        description: Option<String>,
        rule_type: RuleType,
        config: Value,
        threshold_value: f64,
        threshold_operator: ThresholdOperator,
        time_window_seconds: i32,
        is_enabled: bool,
        last_evaluated_at: Option<DateTime<Utc>>,
        last_triggered_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        created_by: UserId,
        channel_ids: Vec<String>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            description,
            rule_type,
            config,
            threshold_value,
            threshold_operator,
            time_window_seconds,
            is_enabled,
            last_evaluated_at,
            last_triggered_at,
            created_at,
            updated_at,
            created_by,
            channel_ids,
        }
    }

    // Getters
    pub fn id(&self) -> &AlertRuleId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn rule_type(&self) -> &RuleType {
        &self.rule_type
    }

    pub fn config(&self) -> &Value {
        &self.config
    }

    pub fn threshold_value(&self) -> f64 {
        self.threshold_value
    }

    pub fn threshold_operator(&self) -> &ThresholdOperator {
        &self.threshold_operator
    }

    pub fn time_window_seconds(&self) -> i32 {
        self.time_window_seconds
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn last_evaluated_at(&self) -> Option<DateTime<Utc>> {
        self.last_evaluated_at
    }

    pub fn last_triggered_at(&self) -> Option<DateTime<Utc>> {
        self.last_triggered_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn created_by(&self) -> &UserId {
        &self.created_by
    }

    pub fn channel_ids(&self) -> &[String] {
        &self.channel_ids
    }

    // Mutators
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn update_config(&mut self, config: Value) {
        self.config = config;
        self.updated_at = Utc::now();
    }

    pub fn update_threshold(&mut self, value: f64, operator: ThresholdOperator) {
        self.threshold_value = value;
        self.threshold_operator = operator;
        self.updated_at = Utc::now();
    }

    pub fn update_time_window(&mut self, seconds: i32) {
        self.time_window_seconds = seconds;
        self.updated_at = Utc::now();
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
        self.updated_at = Utc::now();
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
        self.updated_at = Utc::now();
    }

    pub fn mark_evaluated(&mut self) {
        self.last_evaluated_at = Some(Utc::now());
    }

    pub fn mark_triggered(&mut self) {
        self.last_triggered_at = Some(Utc::now());
    }

    pub fn set_channel_ids(&mut self, channel_ids: Vec<String>) {
        self.channel_ids = channel_ids;
        self.updated_at = Utc::now();
    }

    /// Check if the threshold condition is met
    pub fn evaluate(&self, actual_value: f64) -> bool {
        self.threshold_operator
            .evaluate(actual_value, self.threshold_value)
    }
}
