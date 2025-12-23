use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use super::models::AlertRow;
use crate::modules::alerts::domain::{
    Alert, AlertDomainError, AlertId, AlertRepository, AlertRuleId, AlertStatus,
};
use crate::modules::projects::domain::ProjectId;

pub struct PostgresAlertRepository {
    pool: Arc<PgPool>,
}

impl PostgresAlertRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_entity(&self, row: AlertRow) -> Alert {
        Alert::from_db(
            AlertId::new(row.id.to_string()),
            AlertRuleId::new(row.rule_id.to_string()),
            ProjectId::new(row.project_id.to_string()),
            AlertStatus::from_str(&row.status),
            row.triggered_at,
            row.resolved_at,
            row.trigger_value,
            row.message,
            row.metadata,
        )
    }
}

#[async_trait]
impl AlertRepository for PostgresAlertRepository {
    async fn save(&self, alert: &Alert) -> Result<(), AlertDomainError> {
        let id = Uuid::parse_str(alert.id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
        let rule_id = Uuid::parse_str(alert.rule_id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
        let project_id = Uuid::parse_str(alert.project_id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO alerts (
                id, rule_id, project_id, status, triggered_at,
                resolved_at, trigger_value, message, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind(rule_id)
        .bind(project_id)
        .bind(alert.status().as_str())
        .bind(alert.triggered_at())
        .bind(alert.resolved_at())
        .bind(alert.trigger_value())
        .bind(alert.message())
        .bind(alert.metadata())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &AlertId) -> Result<Option<Alert>, AlertDomainError> {
        let uuid = Uuid::parse_str(id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let row: Option<AlertRow> = sqlx::query_as(r#"SELECT * FROM alerts WHERE id = $1"#)
            .bind(uuid)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_entity(r)))
    }

    async fn find_by_project(
        &self,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Alert>, AlertDomainError> {
        let uuid = Uuid::parse_str(project_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let rows: Vec<AlertRow> = sqlx::query_as(
            r#"SELECT * FROM alerts WHERE project_id = $1 ORDER BY triggered_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(uuid)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| self.row_to_entity(r)).collect())
    }

    async fn find_by_rule(
        &self,
        rule_id: &AlertRuleId,
        limit: i64,
    ) -> Result<Vec<Alert>, AlertDomainError> {
        let uuid = Uuid::parse_str(rule_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let rows: Vec<AlertRow> = sqlx::query_as(
            r#"SELECT * FROM alerts WHERE rule_id = $1 ORDER BY triggered_at DESC LIMIT $2"#,
        )
        .bind(uuid)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| self.row_to_entity(r)).collect())
    }

    async fn find_firing_by_rule(
        &self,
        rule_id: &AlertRuleId,
    ) -> Result<Option<Alert>, AlertDomainError> {
        let uuid = Uuid::parse_str(rule_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let row: Option<AlertRow> = sqlx::query_as(
            r#"SELECT * FROM alerts WHERE rule_id = $1 AND status = 'firing' ORDER BY triggered_at DESC LIMIT 1"#,
        )
        .bind(uuid)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_entity(r)))
    }

    async fn update(&self, alert: &Alert) -> Result<(), AlertDomainError> {
        let id = Uuid::parse_str(alert.id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE alerts SET
                status = $2,
                resolved_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(alert.status().as_str())
        .bind(alert.resolved_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn count_by_project(&self, project_id: &ProjectId) -> Result<i64, AlertDomainError> {
        let uuid = Uuid::parse_str(project_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let count: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM alerts WHERE project_id = $1"#)
                .bind(uuid)
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(count)
    }
}
