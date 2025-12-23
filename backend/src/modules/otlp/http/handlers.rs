//! OTLP HTTP handlers

use axum::{
    body::Bytes,
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    Extension, Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::logging::application::dto::IngestLogsCommand;
use crate::modules::logging::application::LogService;
use crate::modules::logging::domain::{LogDomainError, LogRepository};
use crate::modules::logging::infrastructure::ApiKeyContext;
use crate::modules::metrics::application::dto::IngestMetricsCommand;
use crate::modules::metrics::application::MetricsService;
use crate::modules::metrics::domain::{MetricsDomainError, MetricsRepository};
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::otlp::conversion::{convert_otlp_logs, convert_otlp_metrics, convert_otlp_traces};
use crate::modules::otlp::types::logs::{ExportLogsServiceRequest, ExportLogsServiceResponse};
use crate::modules::otlp::types::metrics::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use crate::modules::otlp::types::traces::{ExportTraceServiceRequest, ExportTraceServiceResponse};
use crate::modules::projects::domain::ProjectRepository;
use crate::modules::traces::application::dto::IngestSpansCommand;
use crate::modules::traces::application::TraceService;
use crate::modules::traces::domain::{SpansRepository, TracesDomainError};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// Logs Handler
// ============================================================================

pub async fn ingest_otlp_logs<LR, PR, OMR, ID>(
    State(service): State<Arc<LogService<LR, PR, OMR, ID>>>,
    Extension(ctx): Extension<ApiKeyContext>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ExportLogsServiceResponse>, (StatusCode, Json<ErrorResponse>)>
where
    LR: LogRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    // Parse the request based on content type
    let request: ExportLogsServiceRequest = if content_type.contains("application/x-protobuf") {
        // TODO: Add protobuf decoding when needed
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ErrorResponse {
                error: "Protobuf format not yet supported. Use application/json".to_string(),
                code: "UNSUPPORTED_FORMAT".to_string(),
            }),
        ));
    } else {
        serde_json::from_slice(&body).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid JSON: {}", e),
                    code: "INVALID_JSON".to_string(),
                }),
            )
        })?
    };

    // Convert OTLP logs to internal format
    let logs = convert_otlp_logs(request);
    let log_count = logs.len();

    if logs.is_empty() {
        return Ok(Json(ExportLogsServiceResponse {
            partial_success: None,
        }));
    }

    // Ingest logs
    let cmd = IngestLogsCommand {
        project_id: ctx.project_id.as_str().to_string(),
        logs,
    };

    let result = service.ingest(cmd).await.map_err(|e| {
        let (status, msg) = match e {
            LogDomainError::ProjectNotFound | LogDomainError::ProjectDeleted => {
                (StatusCode::NOT_FOUND, "Project not found")
            }
            LogDomainError::InternalError(ref msg) => {
                tracing::error!(error = %msg, "Internal error during OTLP log ingestion");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            }
            _ => (StatusCode::BAD_REQUEST, "Invalid request"),
        };
        (
            status,
            Json(ErrorResponse {
                error: msg.to_string(),
                code: "INGESTION_ERROR".to_string(),
            }),
        )
    })?;

    // Build response
    let rejected = log_count as i64 - result.accepted as i64;
    let response = if rejected > 0 {
        ExportLogsServiceResponse {
            partial_success: Some(crate::modules::otlp::types::logs::ExportLogsPartialSuccess {
                rejected_log_records: rejected,
                error_message: result.errors.join("; "),
            }),
        }
    } else {
        ExportLogsServiceResponse {
            partial_success: None,
        }
    };

    Ok(Json(response))
}

// ============================================================================
// Metrics Handler
// ============================================================================

pub async fn ingest_otlp_metrics<MR, PR, OMR, ID>(
    State(service): State<Arc<MetricsService<MR, PR, OMR, ID>>>,
    Extension(ctx): Extension<ApiKeyContext>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ExportMetricsServiceResponse>, (StatusCode, Json<ErrorResponse>)>
where
    MR: MetricsRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    let request: ExportMetricsServiceRequest = if content_type.contains("application/x-protobuf") {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ErrorResponse {
                error: "Protobuf format not yet supported. Use application/json".to_string(),
                code: "UNSUPPORTED_FORMAT".to_string(),
            }),
        ));
    } else {
        serde_json::from_slice(&body).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid JSON: {}", e),
                    code: "INVALID_JSON".to_string(),
                }),
            )
        })?
    };

    let metrics = convert_otlp_metrics(request);
    let metric_count = metrics.len();

    if metrics.is_empty() {
        return Ok(Json(ExportMetricsServiceResponse {
            partial_success: None,
        }));
    }

    let cmd = IngestMetricsCommand {
        project_id: ctx.project_id.as_str().to_string(),
        metrics,
    };

    let result = service.ingest(cmd).await.map_err(|e| {
        let (status, msg) = match e {
            MetricsDomainError::ProjectNotFound | MetricsDomainError::ProjectDeleted => {
                (StatusCode::NOT_FOUND, "Project not found")
            }
            MetricsDomainError::InternalError(ref msg) => {
                tracing::error!(error = %msg, "Internal error during OTLP metrics ingestion");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            }
            _ => (StatusCode::BAD_REQUEST, "Invalid request"),
        };
        (
            status,
            Json(ErrorResponse {
                error: msg.to_string(),
                code: "INGESTION_ERROR".to_string(),
            }),
        )
    })?;

    let rejected = metric_count as i64 - result.ingested as i64;
    let response = if rejected > 0 {
        ExportMetricsServiceResponse {
            partial_success: Some(
                crate::modules::otlp::types::metrics::ExportMetricsPartialSuccess {
                    rejected_data_points: rejected,
                    error_message: String::new(),
                },
            ),
        }
    } else {
        ExportMetricsServiceResponse {
            partial_success: None,
        }
    };

    Ok(Json(response))
}

// ============================================================================
// Traces Handler
// ============================================================================

pub async fn ingest_otlp_traces<SR, PR, OMR, ID>(
    State(service): State<Arc<TraceService<SR, PR, OMR, ID>>>,
    Extension(ctx): Extension<ApiKeyContext>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ExportTraceServiceResponse>, (StatusCode, Json<ErrorResponse>)>
where
    SR: SpansRepository,
    PR: ProjectRepository,
    OMR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    let request: ExportTraceServiceRequest = if content_type.contains("application/x-protobuf") {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(ErrorResponse {
                error: "Protobuf format not yet supported. Use application/json".to_string(),
                code: "UNSUPPORTED_FORMAT".to_string(),
            }),
        ));
    } else {
        serde_json::from_slice(&body).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid JSON: {}", e),
                    code: "INVALID_JSON".to_string(),
                }),
            )
        })?
    };

    let spans = convert_otlp_traces(request);
    let span_count = spans.len();

    if spans.is_empty() {
        return Ok(Json(ExportTraceServiceResponse {
            partial_success: None,
        }));
    }

    let cmd = IngestSpansCommand {
        project_id: ctx.project_id.as_str().to_string(),
        spans,
    };

    let result = service.ingest(cmd).await.map_err(|e| {
        let (status, msg) = match e {
            TracesDomainError::ProjectNotFound | TracesDomainError::ProjectDeleted => {
                (StatusCode::NOT_FOUND, "Project not found")
            }
            TracesDomainError::InternalError(ref msg) => {
                tracing::error!(error = %msg, "Internal error during OTLP traces ingestion");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
            }
            _ => (StatusCode::BAD_REQUEST, "Invalid request"),
        };
        (
            status,
            Json(ErrorResponse {
                error: msg.to_string(),
                code: "INGESTION_ERROR".to_string(),
            }),
        )
    })?;

    let rejected = span_count as i64 - result.ingested as i64;
    let response = if rejected > 0 {
        ExportTraceServiceResponse {
            partial_success: Some(
                crate::modules::otlp::types::traces::ExportTracePartialSuccess {
                    rejected_spans: rejected,
                    error_message: String::new(),
                },
            ),
        }
    } else {
        ExportTraceServiceResponse {
            partial_success: None,
        }
    };

    Ok(Json(response))
}
