use async_trait::async_trait;

use super::entity::OrgActivity;
use crate::modules::organizations::domain::errors::OrgDomainError;
use crate::modules::organizations::domain::organization::OrgId;

/// Repository trait for OrgActivity persistence
#[async_trait]
pub trait OrgActivityRepository: Send + Sync {
    /// Save a new activity log entry
    async fn save(&self, activity: &OrgActivity) -> Result<(), OrgDomainError>;

    /// Find activities by organization ID (paginated, most recent first)
    async fn find_by_org(
        &self,
        org_id: &OrgId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<OrgActivity>, OrgDomainError>;
}
