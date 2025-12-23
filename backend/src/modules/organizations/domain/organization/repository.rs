use async_trait::async_trait;

use super::entity::Organization;
use super::value_objects::OrgId;
use crate::modules::organizations::domain::errors::OrgDomainError;

/// Repository trait for Organization persistence
#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    /// Find organization by ID
    async fn find_by_id(&self, id: &OrgId) -> Result<Option<Organization>, OrgDomainError>;

    /// Find organization by slug (case-insensitive)
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, OrgDomainError>;

    /// Save organization (insert or update)
    async fn save(&self, org: &Organization) -> Result<(), OrgDomainError>;

    /// Check if slug exists (for uniqueness validation)
    async fn slug_exists(&self, slug: &str) -> Result<bool, OrgDomainError>;
}
