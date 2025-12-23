use std::sync::Arc;

use crate::modules::alerts::application::dto::{
    AlertChannelResponse, CreateAlertChannelRequest, UpdateAlertChannelRequest,
};
use crate::modules::alerts::domain::{
    AlertChannel, AlertChannelId, AlertChannelRepository, AlertDomainError, ChannelType,
};
use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::{OrgId, OrganizationMemberRepository};
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

pub struct AlertChannelService<CR, PR, MR, ID>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    channel_repo: Arc<CR>,
    project_repo: Arc<PR>,
    member_repo: Arc<MR>,
    id_generator: Arc<ID>,
}

impl<CR, PR, MR, ID> AlertChannelService<CR, PR, MR, ID>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        channel_repo: Arc<CR>,
        project_repo: Arc<PR>,
        member_repo: Arc<MR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            channel_repo,
            project_repo,
            member_repo,
            id_generator,
        }
    }

    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
    ) -> Result<(), AlertDomainError> {
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?
            .ok_or(AlertDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(AlertDomainError::ProjectNotFound);
        }

        let user_id = UserId::new(user_id.to_string());
        let org_id = OrgId::new(project.organization_id().as_str().to_string());

        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await
            .map_err(|e| AlertDomainError::InternalError(e.to_string()))?;

        if membership.is_none() {
            return Err(AlertDomainError::NotAuthorized);
        }

        Ok(())
    }

    fn to_response(&self, channel: &AlertChannel) -> AlertChannelResponse {
        AlertChannelResponse {
            id: channel.id().as_str().to_string(),
            project_id: channel.project_id().as_str().to_string(),
            name: channel.name().to_string(),
            channel_type: channel.channel_type().as_str().to_string(),
            config: channel.config().clone(),
            is_enabled: channel.is_enabled(),
            created_at: channel.created_at(),
            updated_at: channel.updated_at(),
        }
    }

    pub async fn create_channel(
        &self,
        project_id: &str,
        request: CreateAlertChannelRequest,
        user_id: &str,
    ) -> Result<AlertChannelResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        // Validate channel type
        let channel_type = ChannelType::from_str(&request.channel_type)?;

        // Validate name uniqueness
        if self
            .channel_repo
            .name_exists(&project_id, &request.name, None)
            .await?
        {
            return Err(AlertDomainError::ChannelNameExists(request.name));
        }

        // Validate webhook config
        if matches!(channel_type, ChannelType::Webhook) {
            if request.config.get("url").is_none() {
                return Err(AlertDomainError::InvalidChannelConfig(
                    "Webhook channel requires 'url' in config".to_string(),
                ));
            }
        }

        let channel = AlertChannel::new(
            AlertChannelId::new(self.id_generator.generate()),
            project_id,
            request.name,
            channel_type,
            request.config,
        );

        self.channel_repo.save(&channel).await?;

        Ok(self.to_response(&channel))
    }

    pub async fn get_channel(
        &self,
        project_id: &str,
        channel_id: &str,
        user_id: &str,
    ) -> Result<AlertChannelResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let channel_id = AlertChannelId::new(channel_id.to_string());
        let channel = self
            .channel_repo
            .find_by_id(&channel_id)
            .await?
            .ok_or(AlertDomainError::ChannelNotFound)?;

        // Verify channel belongs to project
        if channel.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::ChannelNotFound);
        }

        Ok(self.to_response(&channel))
    }

    pub async fn list_channels(
        &self,
        project_id: &str,
        user_id: &str,
    ) -> Result<Vec<AlertChannelResponse>, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let channels = self.channel_repo.find_by_project(&project_id).await?;

        Ok(channels.iter().map(|c| self.to_response(c)).collect())
    }

    pub async fn update_channel(
        &self,
        project_id: &str,
        channel_id: &str,
        request: UpdateAlertChannelRequest,
        user_id: &str,
    ) -> Result<AlertChannelResponse, AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let channel_id = AlertChannelId::new(channel_id.to_string());
        let mut channel = self
            .channel_repo
            .find_by_id(&channel_id)
            .await?
            .ok_or(AlertDomainError::ChannelNotFound)?;

        // Verify channel belongs to project
        if channel.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::ChannelNotFound);
        }

        // Update name if provided
        if let Some(name) = request.name {
            if self
                .channel_repo
                .name_exists(&project_id, &name, Some(&channel_id))
                .await?
            {
                return Err(AlertDomainError::ChannelNameExists(name));
            }
            channel.update_name(name);
        }

        // Update config if provided
        if let Some(config) = request.config {
            channel.update_config(config);
        }

        // Update enabled status if provided
        if let Some(is_enabled) = request.is_enabled {
            if is_enabled {
                channel.enable();
            } else {
                channel.disable();
            }
        }

        self.channel_repo.update(&channel).await?;

        Ok(self.to_response(&channel))
    }

    pub async fn delete_channel(
        &self,
        project_id: &str,
        channel_id: &str,
        user_id: &str,
    ) -> Result<(), AlertDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        self.verify_project_access(&project_id, user_id).await?;

        let channel_id = AlertChannelId::new(channel_id.to_string());
        let channel = self
            .channel_repo
            .find_by_id(&channel_id)
            .await?
            .ok_or(AlertDomainError::ChannelNotFound)?;

        // Verify channel belongs to project
        if channel.project_id().as_str() != project_id.as_str() {
            return Err(AlertDomainError::ChannelNotFound);
        }

        self.channel_repo.delete(&channel_id).await?;

        Ok(())
    }
}
