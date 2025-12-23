use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::logging::infrastructure::ApiKeyContext;
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::projects::domain::ProjectRepository;
use crate::modules::traces::application::dto::*;
use crate::modules::traces::application::TraceService;
use crate::modules::traces::domain::{SpansRepository, TracesDomainError};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

fn to_error_response(e: TracesDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        TracesDomainError::InvalidSpanKind(msg)
        | TracesDomainError::InvalidSpanStatus(msg)
        | TracesDomainError::InvalidSpanName(msg)
        | TracesDomainError::InvalidTraceId(msg)
        | TracesDomainError::InvalidSpanId(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        TracesDomainError::TooManySpans(count) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Too many spans in trace: {}", count),
                code: "TOO_MANY_SPANS".to_string(),
            }),
        ),
        TracesDomainError::TooManyAttributes(count) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Too many attributes: {}", count),
                code: "TOO_MANY_ATTRIBUTES".to_string(),
            }),
        ),
        TracesDomainError::TraceNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Trace not found".to_string(),
                code: "TRACE_NOT_FOUND".to_string(),
            }),
        ),
        TracesDomainError::ProjectNotFound | TracesDomainError::ProjectDeleted => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
                code: "PROJECT_NOT_FOUND".to_string(),
            }),
        ),
        TracesDomainError::NotAuthorized | TracesDomainError::NotOrgMember => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        ),
        TracesDomainError::InternalError(ref msg) => {
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
// Ingest Handler (API Key auth)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct IngestSpansRequest {
    pub spans: Vec<SpanInput>,
}

pub async fn ingest_spans<SR, PR, OMR, ID>(
    State(service): State<Arc<TraceService<SR, PR, OMR, ID>>>,
    Extension(ctx): Extension<ApiKeyContext>,
    Json(request): Json<IngestSpansRequest>,
) -> Result<(StatusCode, Json<IngestSpansResponse>), (StatusCode, Json<ErrorResponse>)>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = IngestSpansCommand {
        project_id: ctx.project_id.as_str().to_string(),
        spans: request.spans,
    };

    let response = service.ingest(cmd).await.map_err(to_error_response)?;

    Ok((StatusCode::CREATED, Json(response)))
}

// ============================================================================
// Query Handlers (JWT auth)
// ============================================================================

pub async fn search_traces<SR, PR, OMR, ID>(
    State(service): State<Arc<TraceService<SR, PR, OMR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Query(filters): Query<TraceQueryFilters>,
) -> Result<Json<TraceSearchResponse>, (StatusCode, Json<ErrorResponse>)>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = SearchTracesCommand {
        project_id,
        filters,
        requesting_user_id: claims.user_id,
    };

    let response = service.search_traces(cmd).await.map_err(to_error_response)?;

    Ok(Json(response))
}

pub async fn get_trace<SR, PR, OMR, ID>(
    State(service): State<Arc<TraceService<SR, PR, OMR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, trace_id)): Path<(String, String)>,
) -> Result<Json<TraceResponse>, (StatusCode, Json<ErrorResponse>)>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = GetTraceCommand {
        project_id,
        trace_id,
        requesting_user_id: claims.user_id,
    };

    let response = service.get_trace(cmd).await.map_err(to_error_response)?;

    Ok(Json(response))
}

pub async fn list_services<SR, PR, OMR, ID>(
    State(service): State<Arc<TraceService<SR, PR, OMR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<ServicesResponse>, (StatusCode, Json<ErrorResponse>)>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = ListServicesCommand {
        project_id,
        requesting_user_id: claims.user_id,
    };

    let response = service.list_services(cmd).await.map_err(to_error_response)?;

    Ok(Json(response))
}
