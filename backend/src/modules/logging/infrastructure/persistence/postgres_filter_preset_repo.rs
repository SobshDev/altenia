use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::FilterPresetRow;
use crate::modules::auth::domain::UserId;
use crate::modules::logging::domain::{
    FilterConfig, FilterPreset, FilterPresetId, FilterPresetName, FilterPresetRepository, LogDomainError,
};
use crate::modules::projects::domain::ProjectId;

pub struct PostgresFilterPresetRepository {
    pool: Arc<PgPool>,
}

impl PostgresFilterPresetRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_entity(row: FilterPresetRow) -> Result<FilterPreset, LogDomainError> {
        let id = FilterPresetId::new(row.id);
        let project_id = ProjectId::new(row.project_id);
        let user_id = UserId::new(row.user_id);
        let name = FilterPresetName::new(row.name)?;

        let filter_config: FilterConfig = serde_json::from_value(row.filter_config)
            .map_err(|e| LogDomainError::InvalidFilterPreset(e.to_string()))?;

        Ok(FilterPreset::reconstruct(
            id,
            project_id,
            user_id,
            name,
            filter_config,
            row.is_default,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[async_trait]
impl FilterPresetRepository for PostgresFilterPresetRepository {
    async fn find_by_id(&self, id: &FilterPresetId) -> Result<Option<FilterPreset>, LogDomainError> {
        let row: Option<FilterPresetRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, user_id, name, filter_config, is_default, created_at, updated_at
            FROM filter_presets
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_project_and_user(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Vec<FilterPreset>, LogDomainError> {
        let rows: Vec<FilterPresetRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, user_id, name, filter_config, is_default, created_at, updated_at
            FROM filter_presets
            WHERE project_id = $1 AND user_id = $2
            ORDER BY is_default DESC, name ASC
            "#,
        )
        .bind(project_id.as_str())
        .bind(user_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        rows.into_iter()
            .map(Self::row_to_entity)
            .collect()
    }

    async fn find_default(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Option<FilterPreset>, LogDomainError> {
        let row: Option<FilterPresetRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, user_id, name, filter_config, is_default, created_at, updated_at
            FROM filter_presets
            WHERE project_id = $1 AND user_id = $2 AND is_default = TRUE
            "#,
        )
        .bind(project_id.as_str())
        .bind(user_id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(Self::row_to_entity(row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, preset: &FilterPreset) -> Result<(), LogDomainError> {
        let filter_config_json = serde_json::to_value(preset.filter_config())
            .map_err(|e| LogDomainError::InvalidFilterPreset(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO filter_presets (id, project_id, user_id, name, filter_config, is_default, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id)
            DO UPDATE SET
                name = EXCLUDED.name,
                filter_config = EXCLUDED.filter_config,
                is_default = EXCLUDED.is_default,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(preset.id().as_str())
        .bind(preset.project_id().as_str())
        .bind(preset.user_id().as_str())
        .bind(preset.name().as_str())
        .bind(&filter_config_json)
        .bind(preset.is_default())
        .bind(preset.created_at())
        .bind(preset.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &FilterPresetId) -> Result<(), LogDomainError> {
        sqlx::query(
            r#"
            DELETE FROM filter_presets
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn clear_default(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<(), LogDomainError> {
        sqlx::query(
            r#"
            UPDATE filter_presets
            SET is_default = FALSE, updated_at = NOW()
            WHERE project_id = $1 AND user_id = $2 AND is_default = TRUE
            "#,
        )
        .bind(project_id.as_str())
        .bind(user_id.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn exists_by_name(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        name: &str,
    ) -> Result<bool, LogDomainError> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM filter_presets
                WHERE project_id = $1 AND user_id = $2 AND LOWER(name) = LOWER($3)
            )
            "#,
        )
        .bind(project_id.as_str())
        .bind(user_id.as_str())
        .bind(name)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(exists)
    }

    async fn exists_by_name_excluding(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        name: &str,
        exclude_id: &FilterPresetId,
    ) -> Result<bool, LogDomainError> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM filter_presets
                WHERE project_id = $1 AND user_id = $2 AND LOWER(name) = LOWER($3) AND id != $4
            )
            "#,
        )
        .bind(project_id.as_str())
        .bind(user_id.as_str())
        .bind(name)
        .bind(exclude_id.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        Ok(exists)
    }
}
