use async_trait::async_trait;

use super::entity::OrganizationInvite;
use super::value_objects::InviteId;
use crate::modules::organizations::domain::OrgDomainError;

#[async_trait]
pub trait OrganizationInviteRepository: Send + Sync {
    /// Save a new invite
    async fn save(&self, invite: &OrganizationInvite) -> Result<(), OrgDomainError>;

    /// Update an existing invite
    async fn update(&self, invite: &OrganizationInvite) -> Result<(), OrgDomainError>;

    /// Find invite by ID
    async fn find_by_id(&self, id: &InviteId) -> Result<Option<OrganizationInvite>, OrgDomainError>;

    /// Find pending invite for org + email combination
    async fn find_pending_by_org_and_email(
        &self,
        org_id: &str,
        email: &str,
    ) -> Result<Option<OrganizationInvite>, OrgDomainError>;

    /// List pending invites for an organization
    async fn list_pending_by_org(
        &self,
        org_id: &str,
    ) -> Result<Vec<OrganizationInvite>, OrgDomainError>;

    /// List pending invites for a user (by user_id)
    async fn list_pending_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<OrganizationInvite>, OrgDomainError>;

    /// Count pending invites for a user
    async fn count_pending_by_user(&self, user_id: &str) -> Result<i64, OrgDomainError>;

    /// Delete an invite
    async fn delete(&self, id: &InviteId) -> Result<(), OrgDomainError>;

    /// Mark expired invites as expired and return count
    async fn mark_expired(&self) -> Result<i64, OrgDomainError>;
}
