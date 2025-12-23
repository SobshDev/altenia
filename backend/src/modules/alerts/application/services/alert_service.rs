use std::sync::Arc;

use crate::modules::alerts::application::dto::{AlertListResponse, AlertResponse};
use crate::modules::alerts::domain::{
    AlertDomainError, AlertId, AlertRepository, AlertRuleId, AlertRuleRepository,
};
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

pub struct AlertService<AR, RR, PR, MR>
where
    AR: AlertRepository,
    RR: AlertRuleRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
{
    alert_repo: Arc<AR>,
    rule_repo: Arc<RR>,
    project_repo: Arc<PR>,
    member_repo: Arc<MR>,
}

impl<AR, RR, PR, MR> AlertService<AR, RR, PR, MR>
where
    AR: AlertRepository,
    RR: AlertRuleRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
{
    pub fn new(
        alert_repo: Arc<AR>,
        rule_repo: Arc<RR>,
        project_repo: Arc<PR>,
        member_repo: Arc<MR>,
    ) -> Self {
        Self {
            alert_repo,
            rule_repo,
            project_repo,
            member_repo,
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

    fn to_response(
        &self,
        alert: &crate::modules::alerts::domain::Alert,
    ) -> AlertResponse {
        AlertResponse {
            id: alert.id().as_str().to_string(),
            rule_id: alert.rule_id().as_str().to_string(),
            project_id: alert.project_id().as_str().to_string(),
            status: alert.status().as_str().to_string(),
            triggered_at: alert.triggered_at(),
            resolved_at: alert.resolved_at(),
            trigger_value: alert.trigger_value(),
            message: alert.message().map(|s| s.to_string()),
            metadata: alert.metadata().cloned(),
        }
    }

    pub async fn list_alerts(
        &self,
        project_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
        user_id: &str,
    ) -> Result<AlertListResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let alerts = self
            .alert_repo
            .find_by_project(&project_id, limit + 1, offset)
            .await?;

        let total = self.alert_repo.count_by_project(&project_id).await?;
        let has_more = alerts.len() as i64 > limit;

        let alerts: Vec<AlertResponse> = alerts
            .into_iter()
            .take(limit as usize)
            .map(|a| self.to_response(&a))
            .collect();

        Ok(AlertListResponse {
            alerts,
            total,
            has_more,
        })
    }

    pub async fn get_alert(
        &self,
        project_id: &str,
        alert_id: &str,
        user_id: &str,
    ) -> Result<AlertResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let alert_id = AlertId::new(alert_id.to_string());
        let alert = self
            .alert_repo
            .find_by_id(&alert_id)
            .await?
            .ok_or(AlertDomainError::AlertNotFound)?;

        // Verify alert belongs to project
        if alert.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::AlertNotFound);
        }

        Ok(self.to_response(&alert))
    }

    pub async fn get_alerts_by_rule(
        &self,
        project_id: &str,
        rule_id: &str,
        limit: Option<i64>,
        user_id: &str,
    ) -> Result<Vec<AlertResponse>, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        // Verify rule exists and belongs to project
        let rule_id = AlertRuleId::new(rule_id.to_string());
        let rule = self
            .rule_repo
            .find_by_id(&rule_id)
            .await?
            .ok_or(AlertDomainError::RuleNotFound)?;

        if rule.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::RuleNotFound);
        }

        let limit = limit.unwrap_or(50).min(100);
        let alerts = self.alert_repo.find_by_rule(&rule_id, limit).await?;

        Ok(alerts.iter().map(|a| self.to_response(a)).collect())
    }

    pub async fn resolve_alert(
        &self,
        project_id: &str,
        alert_id: &str,
        user_id: &str,
    ) -> Result<AlertResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let alert_id = AlertId::new(alert_id.to_string());
        let mut alert = self
            .alert_repo
            .find_by_id(&alert_id)
            .await?
            .ok_or(AlertDomainError::AlertNotFound)?;

        // Verify alert belongs to project
        if alert.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::AlertNotFound);
        }

        if !alert.is_firing() {
            return Err(AlertDomainError::AlertAlreadyResolved);
        }

        alert.resolve();
        self.alert_repo.update(&alert).await?;

        Ok(self.to_response(&alert))
    }
}
