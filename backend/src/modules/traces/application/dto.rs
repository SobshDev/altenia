use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ==================== Ingest Commands ====================

/// Single span input for ingestion
#[derive(Debug, Clone, Deserialize)]
pub struct SpanInput {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub status_message: Option<String>,
    pub service_name: Option<String>,
    pub service_version: Option<String>,
    #[serde(default)]
    pub resource_attributes: Value,
    #[serde(default)]
    pub attributes: Value,
    #[serde(default)]
    pub events: Vec<SpanEventInput>,
    #[serde(default)]
    pub links: Vec<SpanLinkInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpanEventInput {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub attributes: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpanLinkInput {
    pub trace_id: String,
    pub span_id: String,
    #[serde(default)]
    pub attributes: Value,
}

/// Command to ingest spans
#[derive(Debug, Clone)]
pub struct IngestSpansCommand {
    pub project_id: String,
    pub spans: Vec<SpanInput>,
}

// ==================== Query Commands ====================

/// Filters for trace queries
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TraceQueryFilters {
    pub service_name: Option<String>,
    pub span_name: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_duration_ms: Option<i64>,
    pub max_duration_ms: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Command to search traces
#[derive(Debug, Clone)]
pub struct SearchTracesCommand {
    pub project_id: String,
    pub filters: TraceQueryFilters,
    pub requesting_user_id: String,
}

/// Command to get a specific trace
#[derive(Debug, Clone)]
pub struct GetTraceCommand {
    pub project_id: String,
    pub trace_id: String,
    pub requesting_user_id: String,
}

/// Command to list service names
#[derive(Debug, Clone)]
pub struct ListServicesCommand {
    pub project_id: String,
    pub requesting_user_id: String,
}

// ==================== Responses ====================

/// Response for ingested spans
#[derive(Debug, Clone, Serialize)]
pub struct IngestSpansResponse {
    pub ingested: u32,
}

/// Span response for API
#[derive(Debug, Clone, Serialize)]
pub struct SpanResponse {
    pub id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<f64>,
    pub status: String,
    pub status_message: Option<String>,
    pub service_name: Option<String>,
    pub service_version: Option<String>,
    pub resource_attributes: Value,
    pub attributes: Value,
    pub events: Vec<SpanEventResponse>,
    pub links: Vec<SpanLinkResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanEventResponse {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanLinkResponse {
    pub trace_id: String,
    pub span_id: String,
    pub attributes: Value,
}

/// Trace summary for listing
#[derive(Debug, Clone, Serialize)]
pub struct TraceSummaryResponse {
    pub trace_id: String,
    pub root_span_name: Option<String>,
    pub services: Vec<String>,
    pub span_count: i64,
    pub error_count: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<f64>,
}

/// Response for trace search
#[derive(Debug, Clone, Serialize)]
pub struct TraceSearchResponse {
    pub traces: Vec<TraceSummaryResponse>,
    pub total: i64,
}

/// Full trace with all spans
#[derive(Debug, Clone, Serialize)]
pub struct TraceResponse {
    pub trace_id: String,
    pub spans: Vec<SpanResponse>,
    pub services: Vec<String>,
    pub duration_ms: Option<f64>,
}

/// Response for service names list
#[derive(Debug, Clone, Serialize)]
pub struct ServicesResponse {
    pub services: Vec<String>,
}
