use chrono::{Duration, Utc};
use serde_json::json;
use std::sync::Arc;
use tokio::time;

use crate::modules::alerts::application::dto::WebhookPayload;
use crate::modules::alerts::domain::{
    Alert, AlertChannelRepository, AlertDomainError, AlertId, AlertRepository, AlertRule,
    AlertRuleId, AlertRuleRepository, RuleType, ThresholdOperator,
};
use crate::modules::alerts::infrastructure::notifiers::Notifier;
use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::logging::domain::{LogFilters, LogLevel, LogRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

pub struct RuleEvaluator<RR, AR, CR, LR, PR, ID, N>
where
    RR: AlertRuleRepository,
    AR: AlertRepository,
    CR: AlertChannelRepository,
    LR: LogRepository,
    PR: ProjectRepository,
    ID: IdGenerator,
    N: Notifier,
{
    rule_repo: Arc<RR>,
    alert_repo: Arc<AR>,
    channel_repo: Arc<CR>,
    log_repo: Arc<LR>,
    project_repo: Arc<PR>,
    id_generator: Arc<ID>,
    notifier: Arc<N>,
    evaluation_interval_secs: u64,
}

impl<RR, AR, CR, LR, PR, ID, N> RuleEvaluator<RR, AR, CR, LR, PR, ID, N>
where
    RR: AlertRuleRepository + 'static,
    AR: AlertRepository + 'static,
    CR: AlertChannelRepository + 'static,
    LR: LogRepository + 'static,
    PR: ProjectRepository + 'static,
    ID: IdGenerator + 'static,
    N: Notifier + 'static,
{
    pub fn new(
        rule_repo: Arc<RR>,
        alert_repo: Arc<AR>,
        channel_repo: Arc<CR>,
        log_repo: Arc<LR>,
        project_repo: Arc<PR>,
        id_generator: Arc<ID>,
        notifier: Arc<N>,
        evaluation_interval_secs: u64,
    ) -> Self {
        Self {
            rule_repo,
            alert_repo,
            channel_repo,
            log_repo,
            project_repo,
            id_generator,
            notifier,
            evaluation_interval_secs,
        }
    }

    /// Start the evaluation loop (runs forever)
    pub async fn start(self: Arc<Self>) {
        let mut interval = time::interval(time::Duration::from_secs(self.evaluation_interval_secs));

        tracing::info!(
            interval_secs = self.evaluation_interval_secs,
            "Starting alert rule evaluator"
        );

        loop {
            interval.tick().await;

            if let Err(e) = self.evaluate_all_rules().await {
                tracing::error!(error = %e, "Error evaluating alert rules");
            }
        }
    }

    async fn evaluate_all_rules(&self) -> Result<(), AlertDomainError> {
        let rules = self.rule_repo.find_all_enabled().await?;

        tracing::debug!(count = rules.len(), "Evaluating alert rules");

        for rule in rules {
            if let Err(e) = self.evaluate_rule(&rule).await {
                tracing::warn!(
                    rule_id = %rule.id().as_str(),
                    error = %e,
                    "Error evaluating rule"
                );
            }
        }

        Ok(())
    }

    async fn evaluate_rule(&self, rule: &AlertRule) -> Result<(), AlertDomainError> {
        let now = Utc::now();
        let time_window = Duration::seconds(rule.time_window_seconds() as i64);
        let start_time = now - time_window;

        // Evaluate based on rule type
        let (current_value, should_trigger) = match rule.rule_type() {
            RuleType::ErrorRate => self.evaluate_error_rate(rule, start_time).await?,
            RuleType::LogCount => self.evaluate_log_count(rule, start_time).await?,
            RuleType::PatternMatch => self.evaluate_pattern_match(rule, start_time).await?,
        };

        // Update last_evaluated_at
        let mut updated_rule = rule.clone();
        updated_rule.mark_evaluated();
        self.rule_repo.update(&updated_rule).await?;

        if should_trigger {
            // Check if there's already a firing alert for this rule
            let existing_alert = self.alert_repo.find_firing_by_rule(rule.id()).await?;

            if existing_alert.is_none() {
                // Create new alert
                self.trigger_alert(rule, current_value).await?;
            } else {
                tracing::debug!(
                    rule_id = %rule.id().as_str(),
                    "Alert already firing, skipping"
                );
            }
        } else {
            // Check if there's a firing alert that should be resolved
            if let Some(mut alert) = self.alert_repo.find_firing_by_rule(rule.id()).await? {
                tracing::info!(
                    rule_id = %rule.id().as_str(),
                    alert_id = %alert.id().as_str(),
                    "Resolving alert - condition no longer met"
                );
                alert.resolve();
                self.alert_repo.update(&alert).await?;
            }
        }

        Ok(())
    }

    async fn evaluate_error_rate(
        &self,
        rule: &AlertRule,
        start_time: chrono::DateTime<Utc>,
    ) -> Result<(f64, bool), AlertDomainError> {
        // Get error levels from config, default to ["error", "fatal"]
        let error_levels: Vec<String> = rule
            .config()
            .get("levels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| vec!["error".to_string(), "fatal".to_string()]);

        let project_id = rule.project_id().clone();

        // Get total log count in the time window
        let total_filters = LogFilters {
            levels: None,
            start_time: Some(start_time),
            end_time: None,
            source: None,
            search: None,
            trace_id: None,
            metadata_filters: vec![],
        };
        let total_count = self
            .log_repo
            .count(&project_id, &total_filters)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        if total_count == 0 {
            return Ok((0.0, false));
        }

        // Get error log count
        let error_log_levels: Vec<LogLevel> = error_levels
            .iter()
            .filter_map(|l| LogLevel::from_str(l).ok())
            .collect();

        let error_filters = LogFilters {
            levels: Some(error_log_levels),
            start_time: Some(start_time),
            end_time: None,
            source: None,
            search: None,
            trace_id: None,
            metadata_filters: vec![],
        };
        let error_count = self
            .log_repo
            .count(&project_id, &error_filters)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let error_rate = (error_count as f64 / total_count as f64) * 100.0;
        let should_trigger =
            self.compare_threshold(error_rate, rule.threshold_value(), rule.threshold_operator());

        tracing::debug!(
            rule_id = %rule.id().as_str(),
            error_rate = error_rate,
            threshold = rule.threshold_value(),
            should_trigger,
            "Evaluated error rate rule"
        );

        Ok((error_rate, should_trigger))
    }

    async fn evaluate_log_count(
        &self,
        rule: &AlertRule,
        start_time: chrono::DateTime<Utc>,
    ) -> Result<(f64, bool), AlertDomainError> {
        // Get levels from config
        let levels: Option<Vec<LogLevel>> = rule
            .config()
            .get("levels")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| LogLevel::from_str(s).ok())
                    .collect()
            });

        // Get source from config
        let source: Option<String> = rule
            .config()
            .get("source")
            .and_then(|v| v.as_str())
            .map(String::from);

        let project_id = rule.project_id().clone();

        let filters = LogFilters {
            levels,
            start_time: Some(start_time),
            end_time: None,
            source,
            search: None,
            trace_id: None,
            metadata_filters: vec![],
        };

        let count = self
            .log_repo
            .count(&project_id, &filters)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let count_f64 = count as f64;
        let should_trigger =
            self.compare_threshold(count_f64, rule.threshold_value(), rule.threshold_operator());

        tracing::debug!(
            rule_id = %rule.id().as_str(),
            count = count_f64,
            threshold = rule.threshold_value(),
            should_trigger,
            "Evaluated log count rule"
        );

        Ok((count_f64, should_trigger))
    }

    async fn evaluate_pattern_match(
        &self,
        rule: &AlertRule,
        start_time: chrono::DateTime<Utc>,
    ) -> Result<(f64, bool), AlertDomainError> {
        // Get pattern from config
        let pattern: String = rule
            .config()
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        if pattern.is_empty() {
            return Ok((0.0, false));
        }

        let project_id = rule.project_id().clone();

        let filters = LogFilters {
            levels: None,
            start_time: Some(start_time),
            end_time: None,
            source: None,
            search: Some(pattern),
            trace_id: None,
            metadata_filters: vec![],
        };

        let count = self
            .log_repo
            .count(&project_id, &filters)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let count_f64 = count as f64;
        let should_trigger =
            self.compare_threshold(count_f64, rule.threshold_value(), rule.threshold_operator());

        tracing::debug!(
            rule_id = %rule.id().as_str(),
            count = count_f64,
            threshold = rule.threshold_value(),
            should_trigger,
            "Evaluated pattern match rule"
        );

        Ok((count_f64, should_trigger))
    }

    fn compare_threshold(
        &self,
        value: f64,
        threshold: f64,
        operator: &ThresholdOperator,
    ) -> bool {
        match operator {
            ThresholdOperator::GreaterThan => value > threshold,
            ThresholdOperator::GreaterThanOrEqual => value >= threshold,
            ThresholdOperator::LessThan => value < threshold,
            ThresholdOperator::LessThanOrEqual => value <= threshold,
        }
    }

    async fn trigger_alert(
        &self,
        rule: &AlertRule,
        trigger_value: f64,
    ) -> Result<(), AlertDomainError> {
        // Get project name
        let project = self
            .project_repo
            .find_by_id(rule.project_id())
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?
            .ok_or(AlertDomainError::ProjectNotFound)?;

        let message = format!(
            "{} is {:.2}, threshold is {:.2} ({})",
            rule.rule_type().as_str(),
            trigger_value,
            rule.threshold_value(),
            rule.threshold_operator().as_str()
        );

        // Create alert
        let alert = Alert::new(
            AlertId::new(self.id_generator.generate()),
            AlertRuleId::new(rule.id().as_str().to_string()),
            ProjectId::new(rule.project_id().as_str().to_string()),
            trigger_value,
            message.clone(),
            Some(json!({
                "rule_type": rule.rule_type().as_str(),
                "time_window_seconds": rule.time_window_seconds()
            })),
        );

        self.alert_repo.save(&alert).await?;

        // Update rule last_triggered_at
        let mut updated_rule = rule.clone();
        updated_rule.mark_triggered();
        self.rule_repo.update(&updated_rule).await?;

        tracing::info!(
            rule_id = %rule.id().as_str(),
            alert_id = %alert.id().as_str(),
            trigger_value,
            "Alert triggered"
        );

        // Send notifications
        let webhook_payload = WebhookPayload {
            alert_id: alert.id().as_str().to_string(),
            rule_id: rule.id().as_str().to_string(),
            rule_name: rule.name().to_string(),
            project_id: rule.project_id().as_str().to_string(),
            project_name: project.name().as_str().to_string(),
            status: alert.status().as_str().to_string(),
            triggered_at: alert.triggered_at(),
            trigger_value,
            threshold: rule.threshold_value(),
            threshold_operator: rule.threshold_operator().as_str().to_string(),
            message,
            metadata: alert.metadata().cloned(),
        };

        // Get channels and send notifications
        let channels = self.channel_repo.find_by_ids(rule.channel_ids()).await?;
        for channel in channels {
            if let Err(e) = self
                .notifier
                .send(&webhook_payload, channel.config())
                .await
            {
                tracing::warn!(
                    channel_id = %channel.id().as_str(),
                    error = %e,
                    "Failed to send notification"
                );
            }
        }

        Ok(())
    }
}
