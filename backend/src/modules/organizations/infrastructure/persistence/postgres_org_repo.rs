use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::OrganizationRow;
use crate::modules::organizations::domain::{
    OrgDomainError, OrgId, OrgName, OrgSlug, Organization, OrganizationRepository,
};

pub struct PostgresOrganizationRepository {
    pool: Arc<PgPool>,
}

impl PostgresOrganizationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_org(row: OrganizationRow) -> Result<Organization, OrgDomainError> {
        let id = OrgId::new(row.id);
        let name = OrgName::new(row.name)?;
        let slug = OrgSlug::from_string(row.slug)?;

        Ok(Organization::reconstruct(
            id,
            name,
            slug,
            row.is_personal,
            row.created_at,
            row.updated_at,
            row.deleted_at,
        ))
    }
}

#[async_trait]
impl OrganizationRepository for PostgresOrganizationRepository {
    async fn find_by_id(&self, id: &OrgId) -> Result<Option<Organization>, OrgDomainError> {
        let row: Option<OrganizationRow> = sqlx::query_as(
            r#"
            SELECT id, name, slug, is_personal, created_at, updated_at, deleted_at
            FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_org).transpose()
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, OrgDomainError> {
        let row: Option<OrganizationRow> = sqlx::query_as(
            r#"
            SELECT id, name, slug, is_personal, created_at, updated_at, deleted_at
            FROM organizations
            WHERE LOWER(slug) = LOWER($1) AND deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_org).transpose()
    }

    async fn save(&self, org: &Organization) -> Result<(), OrgDomainError> {
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, slug, is_personal, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                updated_at = EXCLUDED.updated_at,
                deleted_at = EXCLUDED.deleted_at
            "#,
        )
        .bind(org.id().as_str())
        .bind(org.name().as_str())
        .bind(org.slug().as_str())
        .bind(org.is_personal())
        .bind(org.created_at())
        .bind(org.updated_at())
        .bind(org.deleted_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn slug_exists(&self, slug: &str) -> Result<bool, OrgDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM organizations
            WHERE LOWER(slug) = LOWER($1) AND deleted_at IS NULL
            "#,
        )
        .bind(slug)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(count.0 > 0)
    }
}
