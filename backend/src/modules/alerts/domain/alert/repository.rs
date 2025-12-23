use async_trait::async_trait;

use super::entity::{Alert, AlertId};
use crate::modules::alerts::domain::alert_rule::AlertRuleId;
use crate::modules::alerts::domain::AlertDomainError;
use crate::modules::projects::domain::ProjectId;

#[async_trait]
pub trait AlertRepository: Send + Sync {
    /// Save a new alert
    async fn save(&self, alert: &Alert) -> Result<(), AlertDomainError>;

    /// Find an alert by ID
    async fn find_by_id(&self, id: &AlertId) -> Result<Option<Alert>, AlertDomainError>;

    /// Find alerts for a project (with pagination)
    async fn find_by_project(
        &self,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Alert>, AlertDomainError>;

    /// Find alerts for a rule
    async fn find_by_rule(
        &self,
        rule_id: &AlertRuleId,
        limit: i64,
    ) -> Result<Vec<Alert>, AlertDomainError>;

    /// Find currently firing alerts for a rule (to avoid duplicates)
    async fn find_firing_by_rule(
        &self,
        rule_id: &AlertRuleId,
    ) -> Result<Option<Alert>, AlertDomainError>;

    /// Update an alert (mainly for resolving)
    async fn update(&self, alert: &Alert) -> Result<(), AlertDomainError>;

    /// Count alerts for a project
    async fn count_by_project(&self, project_id: &ProjectId) -> Result<i64, AlertDomainError>;
}
