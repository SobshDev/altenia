use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::ApiKeyRow;
use crate::modules::projects::domain::{
    ApiKey, ApiKeyId, ApiKeyName, ApiKeyPrefix, ApiKeyRepository, ProjectDomainError, ProjectId,
};

pub struct PostgresApiKeyRepository {
    pool: Arc<PgPool>,
}

impl PostgresApiKeyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_api_key(row: ApiKeyRow) -> Result<ApiKey, ProjectDomainError> {
        let id = ApiKeyId::new(row.id);
        let project_id = ProjectId::new(row.project_id);
        let name = ApiKeyName::new(row.name)?;
        let key_prefix = ApiKeyPrefix::from_string(row.key_prefix);

        Ok(ApiKey::reconstruct(
            id,
            project_id,
            name,
            key_prefix,
            row.key_hash,
            row.created_at,
            row.expires_at,
            row.revoked_at,
        ))
    }
}

#[async_trait]
impl ApiKeyRepository for PostgresApiKeyRepository {
    async fn find_by_id(&self, id: &ApiKeyId) -> Result<Option<ApiKey>, ProjectDomainError> {
        let row: Option<ApiKeyRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, name, key_prefix, key_hash,
                   created_at, expires_at, revoked_at
            FROM api_keys
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_api_key).transpose()
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, ProjectDomainError> {
        let row: Option<ApiKeyRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, name, key_prefix, key_hash,
                   created_at, expires_at, revoked_at
            FROM api_keys
            WHERE key_hash = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(hash)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_api_key).transpose()
    }

    async fn find_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<ApiKey>, ProjectDomainError> {
        let rows: Vec<ApiKeyRow> = sqlx::query_as(
            r#"
            SELECT id, project_id, name, key_prefix, key_hash,
                   created_at, expires_at, revoked_at
            FROM api_keys
            WHERE project_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(project_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_api_key).collect()
    }

    async fn save(&self, api_key: &ApiKey) -> Result<(), ProjectDomainError> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (id, project_id, name, key_prefix, key_hash,
                                  created_at, expires_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                revoked_at = EXCLUDED.revoked_at
            "#,
        )
        .bind(api_key.id().as_str())
        .bind(api_key.project_id().as_str())
        .bind(api_key.name().as_str())
        .bind(api_key.key_prefix().as_str())
        .bind(api_key.key_hash())
        .bind(api_key.created_at())
        .bind(api_key.expires_at())
        .bind(api_key.revoked_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn revoke(&self, id: &ApiKeyId) -> Result<(), ProjectDomainError> {
        sqlx::query(
            r#"
            UPDATE api_keys SET revoked_at = NOW()
            WHERE id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(id.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| ProjectDomainError::InternalError(e.to_string()))?;

        Ok(())
    }
}
