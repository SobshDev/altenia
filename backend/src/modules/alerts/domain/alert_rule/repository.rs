use async_trait::async_trait;

use super::entity::AlertRule;
use super::value_objects::AlertRuleId;
use crate::modules::alerts::domain::AlertDomainError;
use crate::modules::projects::domain::ProjectId;

#[async_trait]
pub trait AlertRuleRepository: Send + Sync {
    /// Save a new alert rule
    async fn save(&self, rule: &AlertRule) -> Result<(), AlertDomainError>;

    /// Find a rule by ID
    async fn find_by_id(&self, id: &AlertRuleId) -> Result<Option<AlertRule>, AlertDomainError>;

    /// Find all rules for a project
    async fn find_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<AlertRule>, AlertDomainError>;

    /// Find all enabled rules (for background evaluation)
    async fn find_all_enabled(&self) -> Result<Vec<AlertRule>, AlertDomainError>;

    /// Update a rule
    async fn update(&self, rule: &AlertRule) -> Result<(), AlertDomainError>;

    /// Delete a rule
    async fn delete(&self, id: &AlertRuleId) -> Result<(), AlertDomainError>;

    /// Check if rule name exists in project
    async fn name_exists(
        &self,
        project_id: &ProjectId,
        name: &str,
        exclude_id: Option<&AlertRuleId>,
    ) -> Result<bool, AlertDomainError>;

    /// Set channel IDs for a rule
    async fn set_channels(
        &self,
        rule_id: &AlertRuleId,
        channel_ids: &[String],
    ) -> Result<(), AlertDomainError>;

    /// Get channel IDs for a rule
    async fn get_channel_ids(
        &self,
        rule_id: &AlertRuleId,
    ) -> Result<Vec<String>, AlertDomainError>;
}
