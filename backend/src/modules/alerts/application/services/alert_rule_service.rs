use std::sync::Arc;

use crate::modules::alerts::application::dto::{
    AlertRuleResponse, CreateAlertRuleRequest, UpdateAlertRuleRequest,
};
use crate::modules::alerts::domain::{
    AlertChannelRepository, AlertDomainError, AlertRule, AlertRuleId, AlertRuleRepository,
    RuleType, ThresholdOperator,
};
use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

pub struct AlertRuleService<RR, CR, PR, MR, ID>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    rule_repo: Arc<RR>,
    channel_repo: Arc<CR>,
    project_repo: Arc<PR>,
    member_repo: Arc<MR>,
    id_generator: Arc<ID>,
}

impl<RR, CR, PR, MR, ID> AlertRuleService<RR, CR, PR, MR, ID>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        rule_repo: Arc<RR>,
        channel_repo: Arc<CR>,
        project_repo: Arc<PR>,
        member_repo: Arc<MR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            rule_repo,
            channel_repo,
            project_repo,
            member_repo,
            id_generator,
        }
    }

    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
    ) -> Result<(), AlertDomainError> {
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?
            .ok_or(AlertDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(AlertDomainError::ProjectNotFound);
        }

        let user_id_obj = UserId::new(user_id.to_string());
        let org_id = OrgId::new(project.organization_id().as_str().to_string());

        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id_obj)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        if membership.is_none() {
            return Err(AlertDomainError::NotAuthorized);
        }

        Ok(())
    }

    fn to_response(&self, rule: &AlertRule) -> AlertRuleResponse {
        AlertRuleResponse {
            id: rule.id().as_str().to_string(),
            project_id: rule.project_id().as_str().to_string(),
            name: rule.name().to_string(),
            description: rule.description().map(|s| s.to_string()),
            rule_type: rule.rule_type().as_str().to_string(),
            config: rule.config().clone(),
            threshold_value: rule.threshold_value(),
            threshold_operator: rule.threshold_operator().as_str().to_string(),
            time_window_seconds: rule.time_window_seconds(),
            is_enabled: rule.is_enabled(),
            last_evaluated_at: rule.last_evaluated_at(),
            last_triggered_at: rule.last_triggered_at(),
            created_at: rule.created_at(),
            updated_at: rule.updated_at(),
            channel_ids: rule.channel_ids().to_vec(),
        }
    }

    pub async fn create_rule(
        &self,
        project_id: &str,
        request: CreateAlertRuleRequest,
        user_id: &str,
    ) -> Result<AlertRuleResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        // Validate rule type
        let rule_type = RuleType::from_str(&request.rule_type)?;

        // Validate threshold operator
        let threshold_operator = ThresholdOperator::from_str(&request.threshold_operator)?;

        // Validate name uniqueness
        if self
            .rule_repo
            .name_exists(&project_id, &request.name, None)
            .await?
        {
            return Err(AlertDomainError::RuleNameExists(request.name));
        }

        // Validate channels exist and belong to project
        if !request.channel_ids.is_empty() {
            let channels = self.channel_repo.find_by_ids(&request.channel_ids).await?;
            for channel in &channels {
                if channel.project_id().as_str() != project_id.as_str() {
                    return Err(AlertDomainError::ChannelNotFound);
                }
            }
            if channels.len() != request.channel_ids.len() {
                return Err(AlertDomainError::ChannelNotFound);
            }
        }

        let mut rule = AlertRule::new(
            AlertRuleId::new(self.id_generator.generate()),
            project_id,
            request.name,
            request.description,
            rule_type,
            request.config,
            request.threshold_value,
            threshold_operator,
            request.time_window_seconds,
            UserId::new(user_id.to_string()),
        );

        // Set channel IDs if provided
        if !request.channel_ids.is_empty() {
            rule.set_channel_ids(request.channel_ids);
        }

        self.rule_repo.save(&rule).await?;

        Ok(self.to_response(&rule))
    }

    pub async fn get_rule(
        &self,
        project_id: &str,
        rule_id: &str,
        user_id: &str,
    ) -> Result<AlertRuleResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let rule_id = AlertRuleId::new(rule_id.to_string());
        let rule = self
            .rule_repo
            .find_by_id(&rule_id)
            .await?
            .ok_or(AlertDomainError::RuleNotFound)?;

        // Verify rule belongs to project
        if rule.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::RuleNotFound);
        }

        Ok(self.to_response(&rule))
    }

    pub async fn list_rules(
        &self,
        project_id: &str,
        user_id: &str,
    ) -> Result<Vec<AlertRuleResponse>, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let rules = self.rule_repo.find_by_project(&project_id).await?;

        Ok(rules.iter().map(|r| self.to_response(r)).collect())
    }

    pub async fn update_rule(
        &self,
        project_id: &str,
        rule_id: &str,
        request: UpdateAlertRuleRequest,
        user_id: &str,
    ) -> Result<AlertRuleResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let rule_id = AlertRuleId::new(rule_id.to_string());
        let mut rule = self
            .rule_repo
            .find_by_id(&rule_id)
            .await?
            .ok_or(AlertDomainError::RuleNotFound)?;

        // Verify rule belongs to project
        if rule.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::RuleNotFound);
        }

        // Update name if provided
        if let Some(name) = request.name {
            if self
                .rule_repo
                .name_exists(&project_id, &name, Some(&rule_id))
                .await?
            {
                return Err(AlertDomainError::RuleNameExists(name));
            }
            rule.update_name(name);
        }

        // Update description if provided
        if let Some(description) = request.description {
            rule.update_description(Some(description));
        }

        // Update config if provided
        if let Some(config) = request.config {
            rule.update_config(config);
        }

        // Update threshold if provided
        if let Some(threshold_value) = request.threshold_value {
            let operator = if let Some(op) = &request.threshold_operator {
                ThresholdOperator::from_str(op)?
            } else {
                rule.threshold_operator().clone()
            };
            rule.update_threshold(threshold_value, operator);
        } else if let Some(op) = request.threshold_operator {
            let operator = ThresholdOperator::from_str(&op)?;
            rule.update_threshold(rule.threshold_value(), operator);
        }

        // Update time window if provided
        if let Some(time_window) = request.time_window_seconds {
            rule.update_time_window(time_window);
        }

        // Update enabled status if provided
        if let Some(is_enabled) = request.is_enabled {
            if is_enabled {
                rule.enable();
            } else {
                rule.disable();
            }
        }

        // Update channels if provided
        if let Some(channel_ids) = request.channel_ids {
            // Validate channels exist and belong to project
            if !channel_ids.is_empty() {
                let channels = self.channel_repo.find_by_ids(&channel_ids).await?;
                for channel in &channels {
                    if channel.project_id().as_str() != project_id.as_str() {
                        return Err(AlertDomainError::ChannelNotFound);
                    }
                }
                if channels.len() != channel_ids.len() {
                    return Err(AlertDomainError::ChannelNotFound);
                }
            }
            rule.set_channel_ids(channel_ids);
            // Update channels in the junction table
            self.rule_repo
                .set_channels(rule.id(), rule.channel_ids())
                .await?;
        }

        self.rule_repo.update(&rule).await?;

        Ok(self.to_response(&rule))
    }

    pub async fn delete_rule(
        &self,
        project_id: &str,
        rule_id: &str,
        user_id: &str,
    ) -> Result<(), AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let rule_id = AlertRuleId::new(rule_id.to_string());
        let rule = self
            .rule_repo
            .find_by_id(&rule_id)
            .await?
            .ok_or(AlertDomainError::RuleNotFound)?;

        // Verify rule belongs to project
        if rule.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::RuleNotFound);
        }

        self.rule_repo.delete(&rule_id).await?;

        Ok(())
    }
}
