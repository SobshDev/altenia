use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use super::models::{AlertRuleRow, RuleChannelRow};
use crate::modules::alerts::domain::{
    AlertDomainError, AlertRule, AlertRuleId, AlertRuleRepository, RuleType, ThresholdOperator,
};
use crate::modules::auth::domain::UserId;
use crate::modules::projects::domain::ProjectId;

pub struct PostgresAlertRuleRepository {
    pool: Arc<PgPool>,
}

impl PostgresAlertRuleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_entity(&self, row: AlertRuleRow, channel_ids: Vec<String>) -> AlertRule {
        AlertRule::from_db(
            AlertRuleId::new(row.id.to_string()),
            ProjectId::new(row.project_id.to_string()),
            row.name,
            row.description,
            RuleType::from_str(&row.rule_type).unwrap_or(RuleType::ErrorRate),
            row.config,
            row.threshold_value,
            ThresholdOperator::from_str(&row.threshold_operator)
                .unwrap_or(ThresholdOperator::GreaterThan),
            row.time_window_seconds,
            row.is_enabled,
            row.last_evaluated_at,
            row.last_triggered_at,
            row.created_at,
            row.updated_at,
            UserId::new(row.created_by.to_string()),
            channel_ids,
        )
    }
}

#[async_trait]
impl AlertRuleRepository for PostgresAlertRuleRepository {
    async fn save(&self, rule: &AlertRule) -> Result<(), AlertDomainError> {
        let id = Uuid::parse_str(rule.id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
        let project_id = Uuid::parse_str(rule.project_id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
        let created_by = Uuid::parse_str(rule.created_by().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO alert_rules (
                id, project_id, name, description, rule_type, config,
                threshold_value, threshold_operator, time_window_seconds,
                is_enabled, last_evaluated_at, last_triggered_at,
                created_at, updated_at, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(rule.name())
        .bind(rule.description())
        .bind(rule.rule_type().as_str())
        .bind(rule.config())
        .bind(rule.threshold_value())
        .bind(rule.threshold_operator().as_str())
        .bind(rule.time_window_seconds())
        .bind(rule.is_enabled())
        .bind(rule.last_evaluated_at())
        .bind(rule.last_triggered_at())
        .bind(rule.created_at())
        .bind(rule.updated_at())
        .bind(created_by)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        // Save channel associations
        if !rule.channel_ids().is_empty() {
            self.set_channels(rule.id(), rule.channel_ids()).await?;
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &AlertRuleId) -> Result<Option<AlertRule>, AlertDomainError> {
        let uuid = Uuid::parse_str(id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let row: Option<AlertRuleRow> = sqlx::query_as(
            r#"SELECT * FROM alert_rules WHERE id = $1"#,
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        match row {
            Some(r) => {
                let channel_ids = self.get_channel_ids(id).await?;
                Ok(Some(self.row_to_entity(r, channel_ids)))
            }
            None => Ok(None),
        }
    }

    async fn find_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<AlertRule>, AlertDomainError> {
        let uuid = Uuid::parse_str(project_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let rows: Vec<AlertRuleRow> = sqlx::query_as(
            r#"SELECT * FROM alert_rules WHERE project_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(uuid)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            let rule_id = AlertRuleId::new(row.id.to_string());
            let channel_ids = self.get_channel_ids(&rule_id).await?;
            rules.push(self.row_to_entity(row, channel_ids));
        }

        Ok(rules)
    }

    async fn find_all_enabled(&self) -> Result<Vec<AlertRule>, AlertDomainError> {
        let rows: Vec<AlertRuleRow> = sqlx::query_as(
            r#"SELECT * FROM alert_rules WHERE is_enabled = true"#,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let mut rules = Vec::new();
        for row in rows {
            let rule_id = AlertRuleId::new(row.id.to_string());
            let channel_ids = self.get_channel_ids(&rule_id).await?;
            rules.push(self.row_to_entity(row, channel_ids));
        }

        Ok(rules)
    }

    async fn update(&self, rule: &AlertRule) -> Result<(), AlertDomainError> {
        let id = Uuid::parse_str(rule.id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE alert_rules SET
                name = $2,
                description = $3,
                config = $4,
                threshold_value = $5,
                threshold_operator = $6,
                time_window_seconds = $7,
                is_enabled = $8,
                last_evaluated_at = $9,
                last_triggered_at = $10,
                updated_at = $11
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(rule.name())
        .bind(rule.description())
        .bind(rule.config())
        .bind(rule.threshold_value())
        .bind(rule.threshold_operator().as_str())
        .bind(rule.time_window_seconds())
        .bind(rule.is_enabled())
        .bind(rule.last_evaluated_at())
        .bind(rule.last_triggered_at())
        .bind(rule.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &AlertRuleId) -> Result<(), AlertDomainError> {
        let uuid = Uuid::parse_str(id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(r#"DELETE FROM alert_rules WHERE id = $1"#)
            .bind(uuid)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn name_exists(
        &self,
        project_id: &ProjectId,
        name: &str,
        exclude_id: Option<&AlertRuleId>,
    ) -> Result<bool, AlertDomainError> {
        let project_uuid = Uuid::parse_str(project_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let query = match exclude_id {
            Some(id) => {
                let exclude_uuid = Uuid::parse_str(id.as_str())
                    .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
                sqlx::query_scalar::<_, bool>(
                    r#"SELECT EXISTS(SELECT 1 FROM alert_rules WHERE project_id = $1 AND name = $2 AND id != $3)"#,
                )
                .bind(project_uuid)
                .bind(name)
                .bind(exclude_uuid)
            }
            None => {
                sqlx::query_scalar::<_, bool>(
                    r#"SELECT EXISTS(SELECT 1 FROM alert_rules WHERE project_id = $1 AND name = $2)"#,
                )
                .bind(project_uuid)
                .bind(name)
            }
        };

        query
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))
    }

    async fn set_channels(
        &self,
        rule_id: &AlertRuleId,
        channel_ids: &[String],
    ) -> Result<(), AlertDomainError> {
        let rule_uuid = Uuid::parse_str(rule_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        // Delete existing associations
        sqlx::query(r#"DELETE FROM alert_rule_channels WHERE rule_id = $1"#)
            .bind(rule_uuid)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        // Insert new associations
        for channel_id in channel_ids {
            let channel_uuid = Uuid::parse_str(channel_id)
                .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

            sqlx::query(
                r#"INSERT INTO alert_rule_channels (rule_id, channel_id) VALUES ($1, $2)"#,
            )
            .bind(rule_uuid)
            .bind(channel_uuid)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
        }

        Ok(())
    }

    async fn get_channel_ids(
        &self,
        rule_id: &AlertRuleId,
    ) -> Result<Vec<String>, AlertDomainError> {
        let rule_uuid = Uuid::parse_str(rule_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let rows: Vec<RuleChannelRow> = sqlx::query_as(
            r#"SELECT rule_id, channel_id FROM alert_rule_channels WHERE rule_id = $1"#,
        )
        .bind(rule_uuid)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.channel_id.to_string()).collect())
    }
}
