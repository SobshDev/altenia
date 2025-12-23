use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserId;
use crate::modules::logging::application::dto::*;
use crate::modules::logging::domain::{
    FilterConfig, FilterPreset, FilterPresetId, FilterPresetName, FilterPresetRepository,
    LogDomainError, LogLevel, MetadataFilter, MetadataOperator, SortOrder,
};
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::organizations::domain::OrgId;
use crate::modules::projects::domain::{ProjectId, ProjectRepository};

/// Filter preset service - orchestrates filter preset use cases
pub struct FilterPresetService<FPR, PR, MR, ID>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    filter_preset_repo: Arc<FPR>,
    project_repo: Arc<PR>,
    member_repo: Arc<MR>,
    id_generator: Arc<ID>,
}

impl<FPR, PR, MR, ID> FilterPresetService<FPR, PR, MR, ID>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    pub fn new(
        filter_preset_repo: Arc<FPR>,
        project_repo: Arc<PR>,
        member_repo: Arc<MR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            filter_preset_repo,
            project_repo,
            member_repo,
            id_generator,
        }
    }

    /// Verify user has access to project via org membership
    async fn verify_project_access(
        &self,
        project_id: &ProjectId,
        user_id: &str,
    ) -> Result<(), LogDomainError> {
        // Get project
        let project = self
            .project_repo
            .find_by_id(project_id)
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?
            .ok_or(LogDomainError::ProjectNotFound)?;

        if project.is_deleted() {
            return Err(LogDomainError::ProjectDeleted);
        }

        // Verify user is member of the org
        let user_id = UserId::new(user_id.to_string());
        let org_id = OrgId::new(project.organization_id().as_str().to_string());

        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await
            .map_err(|e| LogDomainError::InternalError(e.to_string()))?;

        if membership.is_none() {
            return Err(LogDomainError::NotOrgMember);
        }

        Ok(())
    }

    /// Create a new filter preset
    pub async fn create_preset(
        &self,
        cmd: CreateFilterPresetCommand,
    ) -> Result<FilterPresetResponse, LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());
        let user_id = UserId::new(cmd.requesting_user_id.clone());

        // Verify user access
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        // Validate name
        let name = FilterPresetName::new(cmd.name)?;

        // Check name uniqueness
        if self
            .filter_preset_repo
            .exists_by_name(&project_id, &user_id, name.as_str())
            .await?
        {
            return Err(LogDomainError::FilterPresetNameExists);
        }

        // Convert filter config
        let filter_config = Self::convert_filter_config_dto(cmd.filter_config)?;

        // If setting as default, clear existing default
        if cmd.is_default {
            self.filter_preset_repo
                .clear_default(&project_id, &user_id)
                .await?;
        }

        // Create preset
        let preset_id = FilterPresetId::new(self.id_generator.generate());
        let preset = FilterPreset::new(
            preset_id,
            project_id,
            user_id,
            name,
            filter_config,
            cmd.is_default,
        );

        // Save
        self.filter_preset_repo.save(&preset).await?;

        // Return response
        Ok(Self::to_response(&preset))
    }

    /// List filter presets for a project and user
    pub async fn list_presets(
        &self,
        cmd: ListFilterPresetsCommand,
    ) -> Result<Vec<FilterPresetResponse>, LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());
        let user_id = UserId::new(cmd.requesting_user_id.clone());

        // Verify user access
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        // Get presets
        let presets = self
            .filter_preset_repo
            .find_by_project_and_user(&project_id, &user_id)
            .await?;

        Ok(presets.iter().map(Self::to_response).collect())
    }

    /// Get a single filter preset
    pub async fn get_preset(
        &self,
        cmd: GetFilterPresetCommand,
    ) -> Result<FilterPresetResponse, LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());
        let preset_id = FilterPresetId::new(cmd.preset_id);

        // Verify user access
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        // Get preset
        let preset = self
            .filter_preset_repo
            .find_by_id(&preset_id)
            .await?
            .ok_or(LogDomainError::FilterPresetNotFound)?;

        // Verify preset belongs to this project and user
        if preset.project_id().as_str() != project_id.as_str() {
            return Err(LogDomainError::FilterPresetNotFound);
        }
        if preset.user_id().as_str() != cmd.requesting_user_id {
            return Err(LogDomainError::FilterPresetNotFound);
        }

        Ok(Self::to_response(&preset))
    }

    /// Get the default filter preset for a project and user
    pub async fn get_default_preset(
        &self,
        project_id: &str,
        requesting_user_id: &str,
    ) -> Result<Option<FilterPresetResponse>, LogDomainError> {
        let project_id = ProjectId::new(project_id.to_string());
        let user_id = UserId::new(requesting_user_id.to_string());

        // Verify user access
        self.verify_project_access(&project_id, requesting_user_id)
            .await?;

        // Get default preset
        let preset = self
            .filter_preset_repo
            .find_default(&project_id, &user_id)
            .await?;

        Ok(preset.as_ref().map(Self::to_response))
    }

    /// Update a filter preset
    pub async fn update_preset(
        &self,
        cmd: UpdateFilterPresetCommand,
    ) -> Result<FilterPresetResponse, LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());
        let preset_id = FilterPresetId::new(cmd.preset_id);
        let user_id = UserId::new(cmd.requesting_user_id.clone());

        // Verify user access
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        // Get existing preset
        let mut preset = self
            .filter_preset_repo
            .find_by_id(&preset_id)
            .await?
            .ok_or(LogDomainError::FilterPresetNotFound)?;

        // Verify ownership
        if preset.project_id().as_str() != project_id.as_str() {
            return Err(LogDomainError::FilterPresetNotFound);
        }
        if preset.user_id().as_str() != cmd.requesting_user_id {
            return Err(LogDomainError::FilterPresetNotFound);
        }

        // Update name if provided
        if let Some(new_name) = cmd.name {
            let name = FilterPresetName::new(new_name)?;
            // Check uniqueness (excluding current preset)
            if self
                .filter_preset_repo
                .exists_by_name_excluding(&project_id, &user_id, name.as_str(), preset.id())
                .await?
            {
                return Err(LogDomainError::FilterPresetNameExists);
            }
            preset.update_name(name);
        }

        // Update config if provided
        if let Some(config_dto) = cmd.filter_config {
            let filter_config = Self::convert_filter_config_dto(config_dto)?;
            preset.update_config(filter_config);
        }

        // Update default status if provided
        if let Some(is_default) = cmd.is_default {
            if is_default && !preset.is_default() {
                // Clear existing default and set this one
                self.filter_preset_repo
                    .clear_default(&project_id, &user_id)
                    .await?;
                preset.set_as_default();
            } else if !is_default && preset.is_default() {
                preset.unset_default();
            }
        }

        // Save
        self.filter_preset_repo.save(&preset).await?;

        Ok(Self::to_response(&preset))
    }

    /// Delete a filter preset
    pub async fn delete_preset(&self, cmd: DeleteFilterPresetCommand) -> Result<(), LogDomainError> {
        let project_id = ProjectId::new(cmd.project_id.clone());
        let preset_id = FilterPresetId::new(cmd.preset_id);

        // Verify user access
        self.verify_project_access(&project_id, &cmd.requesting_user_id)
            .await?;

        // Get preset to verify ownership
        let preset = self
            .filter_preset_repo
            .find_by_id(&preset_id)
            .await?
            .ok_or(LogDomainError::FilterPresetNotFound)?;

        // Verify ownership
        if preset.project_id().as_str() != project_id.as_str() {
            return Err(LogDomainError::FilterPresetNotFound);
        }
        if preset.user_id().as_str() != cmd.requesting_user_id {
            return Err(LogDomainError::FilterPresetNotFound);
        }

        // Delete
        self.filter_preset_repo.delete(&preset_id).await?;

        Ok(())
    }

    /// Convert FilterConfigDto to domain FilterConfig
    fn convert_filter_config_dto(dto: FilterConfigDto) -> Result<FilterConfig, LogDomainError> {
        // Convert levels
        let levels = dto
            .levels
            .map(|levels| {
                levels
                    .into_iter()
                    .map(|l| LogLevel::from_str(&l))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;

        // Convert metadata filters
        let metadata_filters = dto
            .metadata_filters
            .into_iter()
            .map(|mf| {
                let operator = MetadataOperator::from_str(&mf.operator)?;
                MetadataFilter::new(mf.key, operator, mf.value)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Parse sort order
        let sort_order = match dto.sort.as_deref() {
            Some("asc") | Some("ascending") => SortOrder::Ascending,
            _ => SortOrder::Descending,
        };

        Ok(FilterConfig {
            levels,
            start_time: dto.start_time,
            end_time: dto.end_time,
            source: dto.source,
            search: dto.search,
            trace_id: dto.trace_id,
            metadata_filters,
            sort_order,
        })
    }

    /// Convert domain FilterPreset to response DTO
    fn to_response(preset: &FilterPreset) -> FilterPresetResponse {
        let config = preset.filter_config();

        let metadata_filters: Vec<MetadataFilterDto> = config
            .metadata_filters
            .iter()
            .map(|mf| MetadataFilterDto {
                key: mf.key.clone(),
                operator: mf.operator.as_str().to_string(),
                value: mf.value.clone(),
            })
            .collect();

        let filter_config = FilterConfigDto {
            levels: config.levels.as_ref().map(|levels| {
                levels.iter().map(|l| l.to_string()).collect()
            }),
            start_time: config.start_time,
            end_time: config.end_time,
            source: config.source.clone(),
            search: config.search.clone(),
            trace_id: config.trace_id.clone(),
            metadata_filters,
            sort: Some(match config.sort_order {
                SortOrder::Ascending => "asc".to_string(),
                SortOrder::Descending => "desc".to_string(),
            }),
        };

        FilterPresetResponse {
            id: preset.id().as_str().to_string(),
            project_id: preset.project_id().as_str().to_string(),
            name: preset.name().as_str().to_string(),
            filter_config,
            is_default: preset.is_default(),
            created_at: preset.created_at(),
            updated_at: preset.updated_at(),
        }
    }
}
