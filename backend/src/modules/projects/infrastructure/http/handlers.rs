use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::dto::*;
use crate::modules::projects::application::services::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, ProjectDomainError, ProjectRepository};

// ============================================================================
// Request/Response DTOs for HTTP layer
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponseDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: String,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponseDto {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyCreatedResponseDto {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub plain_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl From<ProjectResponse> for ProjectResponseDto {
    fn from(r: ProjectResponse) -> Self {
        Self {
            id: r.id,
            name: r.name,
            description: r.description,
            org_id: r.org_id,
            retention_days: r.retention_days,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<ApiKeyResponse> for ApiKeyResponseDto {
    fn from(r: ApiKeyResponse) -> Self {
        Self {
            id: r.id,
            name: r.name,
            key_prefix: r.key_prefix,
            created_at: r.created_at,
            expires_at: r.expires_at,
            is_active: r.is_active,
        }
    }
}

impl From<ApiKeyCreatedResponse> for ApiKeyCreatedResponseDto {
    fn from(r: ApiKeyCreatedResponse) -> Self {
        Self {
            id: r.id,
            name: r.name,
            key_prefix: r.key_prefix,
            plain_key: r.plain_key,
            created_at: r.created_at,
            expires_at: r.expires_at,
        }
    }
}

// ============================================================================
// Error handling
// ============================================================================

fn to_error_response(e: ProjectDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        ProjectDomainError::InvalidProjectName(_)
        | ProjectDomainError::InvalidRetentionDays(_)
        | ProjectDomainError::InvalidApiKeyName(_) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        ProjectDomainError::ProjectNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
                code: "PROJECT_NOT_FOUND".to_string(),
            }),
        ),
        ProjectDomainError::ApiKeyNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "API key not found".to_string(),
                code: "API_KEY_NOT_FOUND".to_string(),
            }),
        ),
        ProjectDomainError::ProjectAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Project with this name already exists".to_string(),
                code: "PROJECT_EXISTS".to_string(),
            }),
        ),
        ProjectDomainError::NotOrgMember => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Not a member of this organization".to_string(),
                code: "NOT_ORG_MEMBER".to_string(),
            }),
        ),
        ProjectDomainError::InsufficientPermissions => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        ),
        ProjectDomainError::ApiKeyRevoked => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "API key has been revoked".to_string(),
                code: "API_KEY_REVOKED".to_string(),
            }),
        ),
        ProjectDomainError::ApiKeyExpired => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "API key has expired".to_string(),
                code: "API_KEY_EXPIRED".to_string(),
            }),
        ),
        ProjectDomainError::ApiKeyInvalid => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid API key".to_string(),
                code: "API_KEY_INVALID".to_string(),
            }),
        ),
        ProjectDomainError::ProjectAlreadyDeleted => (
            StatusCode::GONE,
            Json(ErrorResponse {
                error: "Project has been deleted".to_string(),
                code: "PROJECT_DELETED".to_string(),
            }),
        ),
        ProjectDomainError::InternalError(ref msg) => {
            tracing::error!(error = %msg, "Internal error occurred");
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
// Project Handlers
// ============================================================================

/// Create a new project in an organization
pub async fn create_project<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponseDto>), (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = CreateProjectCommand {
        org_id,
        name: req.name,
        description: req.description,
        retention_days: req.retention_days,
        requesting_user_id: claims.user_id,
    };

    service
        .create_project(cmd)
        .await
        .map(|r| (StatusCode::CREATED, Json(r.into())))
        .map_err(to_error_response)
}

/// List all projects in an organization
pub async fn list_projects<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<ProjectResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .list_projects(&org_id, &claims.user_id)
        .await
        .map(|projects| Json(projects.into_iter().map(Into::into).collect()))
        .map_err(to_error_response)
}

/// Get a project by ID
pub async fn get_project<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .get_project(&project_id, &claims.user_id)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Update a project
pub async fn update_project<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = UpdateProjectCommand {
        project_id,
        name: req.name,
        description: req.description,
        retention_days: req.retention_days,
        requesting_user_id: claims.user_id,
    };

    service
        .update_project(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Delete a project
pub async fn delete_project<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = DeleteProjectCommand {
        project_id,
        requesting_user_id: claims.user_id,
    };

    service
        .delete_project(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

// ============================================================================
// API Key Handlers
// ============================================================================

/// Create a new API key for a project
pub async fn create_api_key<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyCreatedResponseDto>), (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = CreateApiKeyCommand {
        project_id,
        name: req.name,
        expires_in_days: req.expires_in_days,
        requesting_user_id: claims.user_id,
    };

    service
        .create_api_key(cmd)
        .await
        .map(|r| (StatusCode::CREATED, Json(r.into())))
        .map_err(to_error_response)
}

/// List all API keys for a project
pub async fn list_api_keys<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<ApiKeyResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .list_api_keys(&project_id, &claims.user_id)
        .await
        .map(|keys| Json(keys.into_iter().map(Into::into).collect()))
        .map_err(to_error_response)
}

/// Revoke an API key
pub async fn revoke_api_key<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, api_key_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = RevokeApiKeyCommand {
        project_id,
        api_key_id,
        requesting_user_id: claims.user_id,
    };

    service
        .revoke_api_key(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}
