use async_trait::async_trait;

use super::entity::AlertChannel;
use super::value_objects::AlertChannelId;
use crate::modules::alerts::domain::AlertDomainError;
use crate::modules::projects::domain::ProjectId;

#[async_trait]
pub trait AlertChannelRepository: Send + Sync {
    /// Save a new channel
    async fn save(&self, channel: &AlertChannel) -> Result<(), AlertDomainError>;

    /// Find a channel by ID
    async fn find_by_id(
        &self,
        id: &AlertChannelId,
    ) -> Result<Option<AlertChannel>, AlertDomainError>;

    /// Find all channels for a project
    async fn find_by_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<AlertChannel>, AlertDomainError>;

    /// Find channels by IDs
    async fn find_by_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<AlertChannel>, AlertDomainError>;

    /// Update a channel
    async fn update(&self, channel: &AlertChannel) -> Result<(), AlertDomainError>;

    /// Delete a channel
    async fn delete(&self, id: &AlertChannelId) -> Result<(), AlertDomainError>;

    /// Check if channel name exists in project
    async fn name_exists(
        &self,
        project_id: &ProjectId,
        name: &str,
        exclude_id: Option<&AlertChannelId>,
    ) -> Result<bool, AlertDomainError>;
}
