use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use super::middleware::ApiKeyContext;
use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::logging::application::dto::*;
use crate::modules::logging::application::services::LogService;
use crate::modules::logging::domain::LogDomainError;
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::projects::domain::ProjectRepository;

// ============================================================================
// Request/Response DTOs for HTTP layer
// ============================================================================

/// Ingestion request body
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub logs: Vec<LogInputDto>,
}

/// Single log entry in ingestion request
#[derive(Debug, Deserialize)]
pub struct LogInputDto {
    pub level: String,
    pub message: String,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub span_id: Option<String>,
}

/// Query parameters for log queries
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    #[serde(default)]
    pub levels: Option<String>, // Comma-separated list
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub sort: Option<String>,
    /// JSON-encoded array of metadata filters: [{"key":"x","operator":"eq","value":"y"}]
    #[serde(default)]
    pub metadata_filters: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponseDto {
    pub accepted: u32,
    pub rejected: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LogResponseDto {
    pub id: String,
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogQueryResponseDto {
    pub logs: Vec<LogResponseDto>,
    pub total: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct LevelCountDto {
    pub level: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct LogStatsResponseDto {
    pub total_count: i64,
    pub counts_by_level: Vec<LevelCountDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_log: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_log: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// Conversions
// ============================================================================

impl From<LogInputDto> for LogInput {
    fn from(dto: LogInputDto) -> Self {
        Self {
            level: dto.level,
            message: dto.message,
            timestamp: dto.timestamp,
            source: dto.source,
            metadata: dto.metadata,
            trace_id: dto.trace_id,
            span_id: dto.span_id,
        }
    }
}

impl From<IngestResponse> for IngestResponseDto {
    fn from(r: IngestResponse) -> Self {
        Self {
            accepted: r.accepted,
            rejected: r.rejected,
            errors: r.errors,
        }
    }
}

impl From<LogResponse> for LogResponseDto {
    fn from(r: LogResponse) -> Self {
        Self {
            id: r.id,
            level: r.level,
            message: r.message,
            timestamp: r.timestamp,
            received_at: r.received_at,
            source: r.source,
            metadata: r.metadata,
            trace_id: r.trace_id,
            span_id: r.span_id,
        }
    }
}

impl From<LogQueryResponse> for LogQueryResponseDto {
    fn from(r: LogQueryResponse) -> Self {
        Self {
            logs: r.logs.into_iter().map(Into::into).collect(),
            total: r.total,
            has_more: r.has_more,
        }
    }
}

impl From<LevelCount> for LevelCountDto {
    fn from(r: LevelCount) -> Self {
        Self {
            level: r.level,
            count: r.count,
        }
    }
}

impl From<LogStatsResponse> for LogStatsResponseDto {
    fn from(r: LogStatsResponse) -> Self {
        Self {
            total_count: r.total_count,
            counts_by_level: r.counts_by_level.into_iter().map(Into::into).collect(),
            oldest_log: r.oldest_log,
            newest_log: r.newest_log,
        }
    }
}

// ============================================================================
// Error handling
// ============================================================================

fn to_error_response(e: LogDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        LogDomainError::InvalidLevel(msg) | LogDomainError::InvalidMessage(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        LogDomainError::InvalidTimestamp(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "INVALID_TIMESTAMP".to_string(),
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
        LogDomainError::ApiKeyInvalid
        | LogDomainError::ApiKeyRevoked
        | LogDomainError::ApiKeyExpired => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired API key".to_string(),
                code: "UNAUTHORIZED".to_string(),
            }),
        ),
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
        LogDomainError::InternalError(ref msg) => {
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
// Ingestion Handlers (API Key Auth)
// ============================================================================

/// Ingest logs (authenticated via API key)
pub async fn ingest_logs<LR, PR, MR, ID>(
    State(service): State<Arc<LogService<LR, PR, MR, ID>>>,
    Extension(ctx): Extension<ApiKeyContext>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    LR: crate::modules::logging::domain::LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let cmd = IngestLogsCommand {
        project_id: ctx.project_id.as_str().to_string(),
        logs: req.logs.into_iter().map(Into::into).collect(),
    };

    service
        .ingest(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

// ============================================================================
// Query Handlers (JWT Auth)
// ============================================================================

/// Query logs for a project
pub async fn query_logs<LR, PR, MR, ID>(
    State(service): State<Arc<LogService<LR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Query(params): Query<LogQueryParams>,
) -> Result<Json<LogQueryResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    LR: crate::modules::logging::domain::LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    // Parse comma-separated levels
    let levels = params.levels.map(|s| {
        s.split(',')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    });

    // Parse JSON-encoded metadata filters
    let metadata_filters = params
        .metadata_filters
        .map(|json_str| {
            serde_json::from_str::<Vec<MetadataFilterInput>>(&json_str)
                .map_err(|e| LogDomainError::InvalidFilterPreset(format!(
                    "Invalid metadata_filters JSON: {}",
                    e
                )))
        })
        .transpose()
        .map_err(to_error_response)?;

    let filters = QueryFilters {
        levels,
        start_time: params.start_time,
        end_time: params.end_time,
        source: params.source,
        search: params.search,
        trace_id: params.trace_id,
        metadata_filters,
        preset_id: None,
    };

    let cmd = QueryLogsCommand {
        project_id,
        filters,
        limit: params.limit,
        offset: params.offset,
        sort: params.sort,
        requesting_user_id: claims.user_id,
    };

    service
        .query(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Get log statistics for a project
pub async fn get_log_stats<LR, PR, MR, ID>(
    State(service): State<Arc<LogService<LR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<LogStatsResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    LR: crate::modules::logging::domain::LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .get_stats(&project_id, &claims.user_id)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Query parameters for metrics
#[derive(Debug, Deserialize)]
pub struct MetricsQueryParams {
    /// Time bucket granularity: minute, hour, day (default: hour)
    #[serde(default)]
    pub bucket: TimeBucket,
    /// Start time for metrics range
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    /// End time for metrics range
    #[serde(default)]
    pub end_time: Option<DateTime<Utc>>,
    /// Limit for top sources (default: 10)
    #[serde(default = "default_top_sources_limit")]
    pub top_sources_limit: i32,
}

fn default_top_sources_limit() -> i32 {
    10
}

/// Get metrics for dashboard charts
pub async fn get_metrics<LR, PR, MR, ID>(
    State(service): State<Arc<LogService<LR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Query(params): Query<MetricsQueryParams>,
) -> Result<Json<MetricsResponse>, (StatusCode, Json<ErrorResponse>)>
where
    LR: crate::modules::logging::domain::LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let query = MetricsQuery {
        project_id,
        bucket: params.bucket,
        start_time: params.start_time,
        end_time: params.end_time,
        top_sources_limit: params.top_sources_limit,
        requesting_user_id: claims.user_id,
    };

    service
        .get_metrics(query)
        .await
        .map(Json)
        .map_err(to_error_response)
}

/// Export logs to a ZIP file
pub async fn export_logs<LR, PR, MR, ID>(
    State(service): State<Arc<LogService<LR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Json(request): Json<ExportLogsRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>
where
    LR: crate::modules::logging::domain::LogRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let bytes = service
        .export_logs(&project_id, request, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    // Generate filename with timestamp
    let filename = format!(
        "logs-export-{}-{}.zip",
        project_id,
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    let content_disposition = format!("attachment; filename=\"{}\"", filename);

    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/zip"),
            ),
            (
                header::CONTENT_DISPOSITION,
                header::HeaderValue::from_str(&content_disposition).unwrap(),
            ),
        ],
        bytes,
    ))
}
