use async_trait::async_trait;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

use super::models::OrgActivityRow;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{
    ActivityId, ActivityType, OrgActivity, OrgActivityRepository, OrgDomainError, OrgId,
};

pub struct PostgresOrgActivityRepository {
    pool: Arc<PgPool>,
}

impl PostgresOrgActivityRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_activity(row: OrgActivityRow) -> Result<OrgActivity, OrgDomainError> {
        let id = ActivityId::new(row.id);
        let org_id = OrgId::new(row.organization_id);
        let activity_type = ActivityType::from_str(&row.activity_type)?;
        let actor_id = UserId::new(row.actor_id);
        let target_id = row.target_id.map(UserId::new);
        let metadata: Option<HashMap<String, String>> = row
            .metadata
            .map(|m| serde_json::from_value(m).unwrap_or_default());

        Ok(OrgActivity::reconstruct(
            id,
            org_id,
            activity_type,
            actor_id,
            target_id,
            metadata,
            row.created_at,
        ))
    }
}

#[async_trait]
impl OrgActivityRepository for PostgresOrgActivityRepository {
    async fn save(&self, activity: &OrgActivity) -> Result<(), OrgDomainError> {
        let metadata_json = activity
            .metadata()
            .map(|m| serde_json::to_value(m).unwrap());

        sqlx::query(
            r#"
            INSERT INTO organization_activities
                (id, organization_id, activity_type, actor_id, target_id, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(activity.id().as_str())
        .bind(activity.organization_id().as_str())
        .bind(activity.activity_type().as_str())
        .bind(activity.actor_id().as_str())
        .bind(activity.target_id().map(|t| t.as_str()))
        .bind(metadata_json)
        .bind(activity.created_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_org(
        &self,
        org_id: &OrgId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OrgActivity>, OrgDomainError> {
        let rows: Vec<OrgActivityRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, activity_type, actor_id, target_id, metadata, created_at
            FROM organization_activities
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(org_id.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_activity).collect()
    }
}
