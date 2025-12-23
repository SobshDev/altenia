use async_trait::async_trait;

use super::entity::ApiKey;
use super::value_objects::ApiKeyId;
use crate::modules::projects::domain::errors::ProjectDomainError;
use crate::modules::projects::domain::project::ProjectId;

/// Repository trait for ApiKey persistence
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Find API key by ID
    async fn find_by_id(&self, id: &ApiKeyId) -> Result<Option<ApiKey>, ProjectDomainError>;

    /// Find API key by hash (for validation during ingestion)
    async fn find_by_hash(&self, hash: &str) -> Result<Option<ApiKey>, ProjectDomainError>;

    /// Find all API keys for a project
    async fn find_by_project(&self, project_id: &ProjectId)
        -> Result<Vec<ApiKey>, ProjectDomainError>;

    /// Save API key (insert or update)
    async fn save(&self, api_key: &ApiKey) -> Result<(), ProjectDomainError>;

    /// Revoke an API key
    async fn revoke(&self, id: &ApiKeyId) -> Result<(), ProjectDomainError>;
}
