use async_trait::async_trait;

use super::entity::OrganizationMember;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::errors::OrgDomainError;
use crate::modules::organizations::domain::organization::{MemberId, OrgId};

/// Repository trait for OrganizationMember persistence
#[async_trait]
pub trait OrganizationMemberRepository: Send + Sync {
    /// Find membership by ID
    async fn find_by_id(&self, id: &MemberId) -> Result<Option<OrganizationMember>, OrgDomainError>;

    /// Find membership by organization and user
    async fn find_by_org_and_user(
        &self,
        org_id: &OrgId,
        user_id: &UserId,
    ) -> Result<Option<OrganizationMember>, OrgDomainError>;

    /// Find all memberships for an organization
    async fn find_all_by_org(
        &self,
        org_id: &OrgId,
    ) -> Result<Vec<OrganizationMember>, OrgDomainError>;

    /// Find all memberships for a user
    async fn find_all_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<OrganizationMember>, OrgDomainError>;

    /// Find the last accessed organization membership for a user
    async fn find_last_accessed_by_user(
        &self,
        user_id: &UserId,
    ) -> Result<Option<OrganizationMember>, OrgDomainError>;

    /// Find the personal organization membership for a user
    async fn find_personal_org_membership(
        &self,
        user_id: &UserId,
    ) -> Result<Option<OrganizationMember>, OrgDomainError>;

    /// Save membership (insert or update)
    async fn save(&self, member: &OrganizationMember) -> Result<(), OrgDomainError>;

    /// Delete membership
    async fn delete(&self, id: &MemberId) -> Result<(), OrgDomainError>;

    /// Count owners in an organization (for last owner protection)
    async fn count_owners(&self, org_id: &OrgId) -> Result<u32, OrgDomainError>;

    /// Count owners with row-level locking (for race condition prevention)
    /// Uses SELECT...FOR UPDATE to prevent concurrent modifications
    async fn count_owners_for_update(&self, org_id: &OrgId) -> Result<u32, OrgDomainError>;
}
