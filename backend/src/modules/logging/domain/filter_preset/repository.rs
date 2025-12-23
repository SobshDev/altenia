use async_trait::async_trait;

use crate::modules::auth::domain::UserId;
use crate::modules::logging::domain::LogDomainError;
use crate::modules::projects::domain::ProjectId;

use super::entity::FilterPreset;
use super::value_objects::FilterPresetId;

/// Repository trait for filter preset persistence
#[async_trait]
pub trait FilterPresetRepository: Send + Sync {
    /// Find a preset by its ID
    async fn find_by_id(
        &self,
        id: &FilterPresetId,
    ) -> Result<Option<FilterPreset>, LogDomainError>;

    /// Find all presets for a user in a project
    async fn find_by_project_and_user(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Vec<FilterPreset>, LogDomainError>;

    /// Find the default preset for a user in a project
    async fn find_default(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<Option<FilterPreset>, LogDomainError>;

    /// Save a preset (insert or update)
    async fn save(&self, preset: &FilterPreset) -> Result<(), LogDomainError>;

    /// Delete a preset by ID
    async fn delete(&self, id: &FilterPresetId) -> Result<(), LogDomainError>;

    /// Clear the default flag for all presets of a user in a project
    async fn clear_default(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
    ) -> Result<(), LogDomainError>;

    /// Check if a preset with the given name exists for the user in the project
    async fn exists_by_name(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        name: &str,
    ) -> Result<bool, LogDomainError>;

    /// Check if a preset with the given name exists, excluding a specific preset ID
    async fn exists_by_name_excluding(
        &self,
        project_id: &ProjectId,
        user_id: &UserId,
        name: &str,
        exclude_id: &FilterPresetId,
    ) -> Result<bool, LogDomainError>;
}
