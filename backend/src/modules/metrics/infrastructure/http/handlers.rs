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
use crate::modules::metrics::application::dto::*;
use crate::modules::metrics::application::MetricsService;
use crate::modules::metrics::domain::{MetricsDomainError, MetricsRepository};
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::projects::domain::ProjectRepository;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

fn to_error_response(e: MetricsDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        MetricsDomainError::InvalidMetricName(msg)
        | MetricsDomainError::InvalidMetricType(msg)
        | MetricsDomainError::InvalidMetricValue(msg)
        | MetricsDomainError::InvalidTimestamp(msg)
        | MetricsDomainError::InvalidHistogramData(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        MetricsDomainError::ProjectNotFound | MetricsDomainError::ProjectDeleted => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
                code: "PROJECT_NOT_FOUND".to_string(),
            }),
        ),
        MetricsDomainError::NotAuthorized | MetricsDomainError::NotOrgMember => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        ),
        MetricsDomainError::InternalError(ref msg) => {
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
pub struct IngestMetricsRequest {
    pub metrics: Vec<MetricInput>,
}

pub async fn ingest_metrics<MR, PR, OMR, ID>(
    State(service): State<Arc<MetricsService<MR, PR, OMR, ID>>>,
    Extension(ctx): Extension<ApiKeyContext>,
    Json(request): Json<IngestMetricsRequest>,
) -> Result<(StatusCode, Json<IngestMetricsResponse>), (StatusCode, Json<ErrorResponse>)>
where
    MR: MetricsRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = IngestMetricsCommand {
        project_id: ctx.project_id.as_str().to_string(),
        metrics: request.metrics,
    };

    let response = service.ingest(cmd).await.map_err(to_error_response)?;

    Ok((StatusCode::CREATED, Json(response)))
}

// ============================================================================
// Query Handlers (JWT auth)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct QueryMetricsParams {
    pub names: Option<String>,
    pub types: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub trace_id: Option<String>,
    pub rollup: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn query_metrics<MR, PR, OMR, ID>(
    State(service): State<Arc<MetricsService<MR, PR, OMR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Query(params): Query<QueryMetricsParams>,
) -> Result<Json<MetricQueryResponse>, (StatusCode, Json<ErrorResponse>)>
where
    MR: MetricsRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let filters = MetricQueryFilters {
        names: params.names.map(|s| s.split(',').map(String::from).collect()),
        types: params.types.map(|s| s.split(',').map(String::from).collect()),
        start_time: params.start_time.and_then(|s| s.parse().ok()),
        end_time: params.end_time.and_then(|s| s.parse().ok()),
        tags: None,
        trace_id: params.trace_id,
        rollup: params.rollup,
        limit: params.limit,
        offset: params.offset,
    };

    let cmd = QueryMetricsCommand {
        project_id,
        filters,
        requesting_user_id: claims.user_id,
    };

    let response = service.query(cmd).await.map_err(to_error_response)?;

    Ok(Json(response))
}

pub async fn list_metric_names<MR, PR, OMR, ID>(
    State(service): State<Arc<MetricsService<MR, PR, OMR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<MetricNamesResponse>, (StatusCode, Json<ErrorResponse>)>
where
    MR: MetricsRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = ListMetricNamesCommand {
        project_id,
        requesting_user_id: claims.user_id,
    };

    let response = service.list_names(cmd).await.map_err(to_error_response)?;

    Ok(Json(response))
}
