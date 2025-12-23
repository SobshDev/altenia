use async_trait::async_trait;

use super::entity::Project;
use super::value_objects::ProjectId;
use crate::modules::organizations::domain::OrgId;
use crate::modules::projects::domain::errors::ProjectDomainError;

/// Repository trait for Project persistence
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Find project by ID
    async fn find_by_id(&self, id: &ProjectId) -> Result<Option<Project>, ProjectDomainError>;

    /// Find all projects in an organization
    async fn find_by_org(&self, org_id: &OrgId) -> Result<Vec<Project>, ProjectDomainError>;

    /// Save project (insert or update)
    async fn save(&self, project: &Project) -> Result<(), ProjectDomainError>;

    /// Check if project name exists in organization (for uniqueness validation)
    async fn exists_by_name_and_org(
        &self,
        name: &str,
        org_id: &OrgId,
    ) -> Result<bool, ProjectDomainError>;

    /// Check if project name exists in organization, excluding a specific project ID
    async fn exists_by_name_and_org_excluding(
        &self,
        name: &str,
        org_id: &OrgId,
        exclude_id: &ProjectId,
    ) -> Result<bool, ProjectDomainError>;
}
