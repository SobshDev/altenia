use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::OrgInviteRow;
use crate::modules::organizations::domain::invite::{InviteId, InviteStatus, OrganizationInvite};
use crate::modules::organizations::domain::{OrgDomainError, OrgRole, OrganizationInviteRepository};

pub struct PostgresInviteRepository {
    pool: Arc<PgPool>,
}

impl PostgresInviteRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_invite(row: OrgInviteRow) -> Result<OrganizationInvite, OrgDomainError> {
        let id = InviteId::new(row.id);
        let role = OrgRole::from_str(&row.role)?;
        let status: InviteStatus = row.status.parse()?;

        Ok(OrganizationInvite::reconstruct(
            id,
            row.organization_id,
            row.inviter_id,
            row.invitee_email,
            row.invitee_id,
            role,
            status,
            row.expires_at,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[async_trait]
impl OrganizationInviteRepository for PostgresInviteRepository {
    async fn save(&self, invite: &OrganizationInvite) -> Result<(), OrgDomainError> {
        sqlx::query(
            r#"
            INSERT INTO organization_invites
                (id, organization_id, inviter_id, invitee_email, invitee_id, role, status, expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(invite.id().as_str())
        .bind(invite.organization_id())
        .bind(invite.inviter_id())
        .bind(invite.invitee_email())
        .bind(invite.invitee_id())
        .bind(invite.role().as_str())
        .bind(invite.status().as_str())
        .bind(invite.expires_at())
        .bind(invite.created_at())
        .bind(invite.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, invite: &OrganizationInvite) -> Result<(), OrgDomainError> {
        sqlx::query(
            r#"
            UPDATE organization_invites
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(invite.id().as_str())
        .bind(invite.status().as_str())
        .bind(invite.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &InviteId) -> Result<Option<OrganizationInvite>, OrgDomainError> {
        let row: Option<OrgInviteRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, inviter_id, invitee_email, invitee_id, role, status, expires_at, created_at, updated_at
            FROM organization_invites
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_invite).transpose()
    }

    async fn find_pending_by_org_and_email(
        &self,
        org_id: &str,
        email: &str,
    ) -> Result<Option<OrganizationInvite>, OrgDomainError> {
        let row: Option<OrgInviteRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, inviter_id, invitee_email, invitee_id, role, status, expires_at, created_at, updated_at
            FROM organization_invites
            WHERE organization_id = $1 AND invitee_email = $2 AND status = 'pending'
            "#,
        )
        .bind(org_id)
        .bind(email)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_invite).transpose()
    }

    async fn list_pending_by_org(
        &self,
        org_id: &str,
    ) -> Result<Vec<OrganizationInvite>, OrgDomainError> {
        let rows: Vec<OrgInviteRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, inviter_id, invitee_email, invitee_id, role, status, expires_at, created_at, updated_at
            FROM organization_invites
            WHERE organization_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_invite).collect()
    }

    async fn list_pending_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<OrganizationInvite>, OrgDomainError> {
        let rows: Vec<OrgInviteRow> = sqlx::query_as(
            r#"
            SELECT id, organization_id, inviter_id, invitee_email, invitee_id, role, status, expires_at, created_at, updated_at
            FROM organization_invites
            WHERE invitee_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        rows.into_iter().map(Self::row_to_invite).collect()
    }

    async fn count_pending_by_user(&self, user_id: &str) -> Result<i64, OrgDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM organization_invites
            WHERE invitee_id = $1 AND status = 'pending'
            "#,
        )
        .bind(user_id)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(count.0)
    }

    async fn delete(&self, id: &InviteId) -> Result<(), OrgDomainError> {
        sqlx::query("DELETE FROM organization_invites WHERE id = $1")
            .bind(id.as_str())
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn mark_expired(&self) -> Result<i64, OrgDomainError> {
        let result = sqlx::query(
            r#"
            UPDATE organization_invites
            SET status = 'expired', updated_at = NOW()
            WHERE status = 'pending' AND expires_at < NOW()
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }
}
