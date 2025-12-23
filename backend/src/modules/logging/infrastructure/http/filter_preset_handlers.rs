use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::logging::application::dto::*;
use crate::modules::logging::application::services::FilterPresetService;
use crate::modules::logging::domain::{FilterPresetRepository, LogDomainError};
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::projects::domain::ProjectRepository;

use super::handlers::ErrorResponse;

// ============================================================================
// Request/Response DTOs for HTTP layer
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePresetRequest {
    pub name: String,
    pub filter_config: FilterConfigDto,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePresetRequest {
    pub name: Option<String>,
    pub filter_config: Option<FilterConfigDto>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct FilterPresetResponseDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub filter_config: FilterConfigDto,
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<FilterPresetResponse> for FilterPresetResponseDto {
    fn from(r: FilterPresetResponse) -> Self {
        Self {
            id: r.id,
            project_id: r.project_id,
            name: r.name,
            filter_config: r.filter_config,
            is_default: r.is_default,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

// ============================================================================
// Error handling
// ============================================================================

fn to_error_response(e: LogDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        LogDomainError::InvalidFilterPreset(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "INVALID_FILTER_PRESET".to_string(),
            }),
        ),
        LogDomainError::FilterPresetNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Filter preset not found".to_string(),
                code: "FILTER_PRESET_NOT_FOUND".to_string(),
            }),
        ),
        LogDomainError::FilterPresetNameExists => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "A filter preset with this name already exists".to_string(),
                code: "FILTER_PRESET_NAME_EXISTS".to_string(),
            }),
        ),
        LogDomainError::ProjectNotFound | LogDomainError::ProjectDeleted => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
                code: "PROJECT_NOT_FOUND".to_string(),
            }),
        ),
        LogDomainError::NotOrgMember | LogDomainError::InsufficientPermissions => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        ),
        LogDomainError::InvalidLevel(msg) | LogDomainError::InvalidMessage(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        _ => {
            tracing::error!(error = ?e, "Internal error occurred");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "An internal error occurred".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
        }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new filter preset
pub async fn create_preset<FPR, PR, MR, ID>(
    State(service): State<Arc<FilterPresetService<FPR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Json(req): Json<CreatePresetRequest>,
) -> Result<(StatusCode, Json<FilterPresetResponseDto>), (StatusCode, Json<ErrorResponse>)>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = CreateFilterPresetCommand {
        project_id,
        name: req.name,
        filter_config: req.filter_config,
        is_default: req.is_default,
        requesting_user_id: claims.user_id,
    };

    service
        .create_preset(cmd)
        .await
        .map(|r| (StatusCode::CREATED, Json(r.into())))
        .map_err(to_error_response)
}

/// List all filter presets for a project
pub async fn list_presets<FPR, PR, MR, ID>(
    State(service): State<Arc<FilterPresetService<FPR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<FilterPresetResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = ListFilterPresetsCommand {
        project_id,
        requesting_user_id: claims.user_id,
    };

    service
        .list_presets(cmd)
        .await
        .map(|presets| Json(presets.into_iter().map(Into::into).collect()))
        .map_err(to_error_response)
}

/// Get a single filter preset
pub async fn get_preset<FPR, PR, MR, ID>(
    State(service): State<Arc<FilterPresetService<FPR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, preset_id)): Path<(String, String)>,
) -> Result<Json<FilterPresetResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = GetFilterPresetCommand {
        preset_id,
        project_id,
        requesting_user_id: claims.user_id,
    };

    service
        .get_preset(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Get the default filter preset for a project
pub async fn get_default_preset<FPR, PR, MR, ID>(
    State(service): State<Arc<FilterPresetService<FPR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<Option<FilterPresetResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .get_default_preset(&project_id, &claims.user_id)
        .await
        .map(|r| Json(r.map(Into::into)))
        .map_err(to_error_response)
}

/// Update a filter preset
pub async fn update_preset<FPR, PR, MR, ID>(
    State(service): State<Arc<FilterPresetService<FPR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, preset_id)): Path<(String, String)>,
    Json(req): Json<UpdatePresetRequest>,
) -> Result<Json<FilterPresetResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = UpdateFilterPresetCommand {
        preset_id,
        project_id,
        name: req.name,
        filter_config: req.filter_config,
        is_default: req.is_default,
        requesting_user_id: claims.user_id,
    };

    service
        .update_preset(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Delete a filter preset
pub async fn delete_preset<FPR, PR, MR, ID>(
    State(service): State<Arc<FilterPresetService<FPR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, preset_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    FPR: FilterPresetRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = DeleteFilterPresetCommand {
        preset_id,
        project_id,
        requesting_user_id: claims.user_id,
    };

    service
        .delete_preset(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}
