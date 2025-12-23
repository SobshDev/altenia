use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::OrganizationMemberRow;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{
    MemberId, OrgDomainError, OrgId, OrgRole, OrganizationMember, OrganizationMemberRepository,
};

pub struct PostgresOrganizationMemberRepository {
    pool: Arc<PgPool>,
}

impl PostgresOrganizationMemberRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_member(row: OrganizationMemberRow) -> Result<OrganizationMember, OrgDomainError> {
        let id = MemberId::new(row.id);
        let org_id = OrgId::new(row.organization_id);
        let user_id = UserId::new(row.user_id);
        let role = OrgRole::from_str(&row.role)?;

        Ok(OrganizationMember::reconstruct(
            id,
            org_id,
            user_id,
            role,
            row.last_accessed_at,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[async_trait]
impl OrganizationMemberRepository for PostgresOrganizationMemberRepository {
    async fn find_by_id(&self, id: &MemberId) -> Result<Option<OrganizationMember>, OrgDomainError> {
        let row: Option<OrganizationMemberRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, user_id, role, last_accessed_at, created_at, updated_at
            FROM organization_members
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_member).transpose()
    }

    async fn find_by_org_and_user(
        &self,
        org_id: &OrgId,
        user_id: &UserId,
    ) -> Result<Option<OrganizationMember>, OrgDomainError> {
        let row: Option<OrganizationMemberRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, user_id, role, last_accessed_at, created_at, updated_at
            FROM organization_members
            WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id.as_str())
        .bind(user_id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_member).transpose()
    }

    async fn find_all_by_org(
        &self,
        org_id: &OrgId,
    ) -> Result<Vec<OrganizationMember>, OrgDomainError> {
        let rows: Vec<OrganizationMemberRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, user_id, role, last_accessed_at, created_at, updated_at
            FROM organization_members
            WHERE organization_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(org_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_member).collect()
    }

    async fn find_all_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<OrganizationMember>, OrgDomainError> {
        let rows: Vec<OrganizationMemberRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, user_id, role, last_accessed_at, created_at, updated_at
            FROM organization_members
            WHERE user_id = $1
            ORDER BY last_accessed_at DESC NULLS LAST
            "#,
        )
        .bind(user_id.as_str())
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_member).collect()
    }

    async fn find_last_accessed_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Option<OrganizationMember>, OrgDomainError> {
        let row: Option<OrganizationMemberRow> = sqlx::query_as(
            r#"
            SELECT om.id, om.organization_id, om.user_id, om.role, om.last_accessed_at, om.created_at, om.updated_at
            FROM organization_members om
            JOIN organizations o ON om.organization_id = o.id
            WHERE om.user_id = $1 AND o.deleted_at IS NULL
            ORDER BY om.last_accessed_at DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(user_id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_member).transpose()
    }

    async fn find_personal_org_membership(
        &self,
        user_id: &UserId,
    ) -> Result<Option<OrganizationMember>, OrgDomainError> {
        let row: Option<OrganizationMemberRow> = sqlx::query_as(
            r#"
            SELECT om.id, om.organization_id, om.user_id, om.role, om.last_accessed_at, om.created_at, om.updated_at
            FROM organization_members om
            JOIN organizations o ON om.organization_id = o.id
            WHERE om.user_id = $1 AND o.is_personal = TRUE AND o.deleted_at IS NULL
            LIMIT 1
            "#,
        )
        .bind(user_id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_member).transpose()
    }

    async fn save(&self, member: &OrganizationMember) -> Result<(), OrgDomainError> {
        sqlx::query(
            r#"
            INSERT INTO organization_members (id, organization_id, user_id, role, last_accessed_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                role = EXCLUDED.role,
                last_accessed_at = EXCLUDED.last_accessed_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(member.id().as_str())
        .bind(member.organization_id().as_str())
        .bind(member.user_id().as_str())
        .bind(member.role().as_str())
        .bind(member.last_accessed_at())
        .bind(member.created_at())
        .bind(member.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &MemberId) -> Result<(), OrgDomainError> {
        sqlx::query(
            r#"
            DELETE FROM organization_members
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn count_owners(&self, org_id: &OrgId) -> Result<u32, OrgDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM organization_members
            WHERE organization_id = $1 AND role = 'owner'
            "#,
        )
        .bind(org_id.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(count.0 as u32)
    }

    async fn count_owners_for_update(&self, org_id: &OrgId) -> Result<u32, OrgDomainError> {
        // Use FOR UPDATE in subquery to lock rows and prevent race conditions
        // PostgreSQL doesn't allow FOR UPDATE with aggregate functions directly
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM (
                SELECT 1 FROM organization_members
                WHERE organization_id = $1 AND role = 'owner'
                FOR UPDATE
            ) AS locked_rows
            "#,
        )
        .bind(org_id.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(count.0 as u32)
    }
}
