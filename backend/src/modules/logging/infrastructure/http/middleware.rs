use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::services::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, Project, ProjectId, ProjectRepository};

/// Context injected after API key validation
#[derive(Debug, Clone)]
pub struct ApiKeyContext {
    pub project_id: ProjectId,
    pub project: Project,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyErrorResponse {
    pub error: String,
    pub code: String,
}

/// Middleware to validate API key for log ingestion
///
/// Extracts API key from:
/// - X-API-Key header
/// - Authorization: Bearer <key>
pub async fn api_key_middleware<PR, AR, OR, MR, ID>(
    State(service): State<Arc<ProjectService<PR, AR, OR, MR, ID>>>,
    mut request: Request<Body>,
    next: Next,
) -> Response
where
    PR: ProjectRepository,
    AR: ApiKeyRepository,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    // Try to extract API key from headers
    let api_key = extract_api_key(&request);

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiKeyErrorResponse {
                    error: "Missing API key. Provide via X-API-Key header or Authorization: Bearer".to_string(),
                    code: "MISSING_API_KEY".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Validate the API key
    match service.validate_api_key(&api_key).await {
        Ok((project_id, project)) => {
            // Inject context into request extensions
            request.extensions_mut().insert(ApiKeyContext {
                project_id,
                project,
            });
            next.run(request).await
        }
        Err(e) => {
            let (status, error, code) = match e {
                crate::modules::projects::domain::ProjectDomainError::ApiKeyInvalid => {
                    (StatusCode::UNAUTHORIZED, "Invalid API key", "INVALID_API_KEY")
                }
                crate::modules::projects::domain::ProjectDomainError::ApiKeyRevoked => {
                    (StatusCode::UNAUTHORIZED, "API key has been revoked", "API_KEY_REVOKED")
                }
                crate::modules::projects::domain::ProjectDomainError::ApiKeyExpired => {
                    (StatusCode::UNAUTHORIZED, "API key has expired", "API_KEY_EXPIRED")
                }
                crate::modules::projects::domain::ProjectDomainError::ProjectNotFound => {
                    (StatusCode::FORBIDDEN, "Project not found or deleted", "PROJECT_NOT_FOUND")
                }
                _ => {
                    tracing::error!(error = %e, "API key validation error");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal error", "INTERNAL_ERROR")
                }
            };

            (
                status,
                Json(ApiKeyErrorResponse {
                    error: error.to_string(),
                    code: code.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Extract API key from request headers
fn extract_api_key(request: &Request<Body>) -> Option<String> {
    // Try X-API-Key header first
    if let Some(key) = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
    {
        return Some(key.to_string());
    }

    // Try Authorization: Bearer header
    if let Some(auth) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(key) = auth.strip_prefix("Bearer ") {
            // Only accept API keys (starting with alt_pk_)
            if key.starts_with("alt_pk_") {
                return Some(key.to_string());
            }
        }
    }

    None
}
