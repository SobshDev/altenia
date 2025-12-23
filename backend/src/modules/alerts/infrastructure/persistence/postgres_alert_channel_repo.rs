use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use super::models::AlertChannelRow;
use crate::modules::alerts::domain::{
    AlertChannel, AlertChannelId, AlertChannelRepository, AlertDomainError, ChannelType,
};
use crate::modules::projects::domain::ProjectId;

pub struct PostgresAlertChannelRepository {
    pool: Arc<PgPool>,
}

impl PostgresAlertChannelRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_entity(&self, row: AlertChannelRow) -> AlertChannel {
        AlertChannel::from_db(
            AlertChannelId::new(row.id.to_string()),
            ProjectId::new(row.project_id.to_string()),
            row.name,
            ChannelType::from_str(&row.channel_type).unwrap_or(ChannelType::Webhook),
            row.config,
            row.is_enabled,
            row.created_at,
            row.updated_at,
        )
    }
}

#[async_trait]
impl AlertChannelRepository for PostgresAlertChannelRepository {
    async fn save(&self, channel: &AlertChannel) -> Result<(), AlertDomainError> {
        let id = Uuid::parse_str(channel.id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
        let project_id = Uuid::parse_str(channel.project_id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO alert_channels (
                id, project_id, name, channel_type, config,
                is_enabled, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(channel.name())
        .bind(channel.channel_type().as_str())
        .bind(channel.config())
        .bind(channel.is_enabled())
        .bind(channel.created_at())
        .bind(channel.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &AlertChannelId,
    ) -> Result<Option<AlertChannel>, AlertDomainError> {
        let uuid = Uuid::parse_str(id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let row: Option<AlertChannelRow> =
            sqlx::query_as(r#"SELECT * FROM alert_channels WHERE id = $1"#)
                .bind(uuid)
                .fetch_optional(self.pool.as_ref())
                .await
                .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_entity(r)))
    }

    async fn find_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<AlertChannel>, AlertDomainError> {
        let uuid = Uuid::parse_str(project_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let rows: Vec<AlertChannelRow> = sqlx::query_as(
            r#"SELECT * FROM alert_channels WHERE project_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(uuid)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| self.row_to_entity(r)).collect())
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<AlertChannel>, AlertDomainError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let uuids: Vec<Uuid> = ids
            .iter()
            .map(|id| {
                Uuid::parse_str(id).map_err(|e| AlertDomainError::InternalError(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let rows: Vec<AlertChannelRow> = sqlx::query_as(
            r#"SELECT * FROM alert_channels WHERE id = ANY($1) AND is_enabled = true"#,
        )
        .bind(&uuids)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| self.row_to_entity(r)).collect())
    }

    async fn update(&self, channel: &AlertChannel) -> Result<(), AlertDomainError> {
        let id = Uuid::parse_str(channel.id().as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE alert_channels SET
                name = $2,
                config = $3,
                is_enabled = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(channel.name())
        .bind(channel.config())
        .bind(channel.is_enabled())
        .bind(channel.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &AlertChannelId) -> Result<(), AlertDomainError> {
        let uuid = Uuid::parse_str(id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        sqlx::query(r#"DELETE FROM alert_channels WHERE id = $1"#)
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
        exclude_id: Option<&AlertChannelId>,
    ) -> Result<bool, AlertDomainError> {
        let project_uuid = Uuid::parse_str(project_id.as_str())
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        let query = match exclude_id {
            Some(id) => {
                let exclude_uuid = Uuid::parse_str(id.as_str())
                    .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;
                sqlx::query_scalar::<_, bool>(
                    r#"SELECT EXISTS(SELECT 1 FROM alert_channels WHERE project_id = $1 AND name = $2 AND id != $3)"#,
                )
                .bind(project_uuid)
                .bind(name)
                .bind(exclude_uuid)
            }
            None => {
                sqlx::query_scalar::<_, bool>(
                    r#"SELECT EXISTS(SELECT 1 FROM alert_channels WHERE project_id = $1 AND name = $2)"#,
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
}
