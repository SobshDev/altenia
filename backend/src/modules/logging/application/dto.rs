use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ==================== Commands ====================

/// Single log entry input for ingestion
#[derive(Debug, Clone, Deserialize)]
pub struct LogInput {
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

/// Command to ingest logs
#[derive(Debug, Clone)]
pub struct IngestLogsCommand {
    pub project_id: String,
    pub logs: Vec<LogInput>,
}

/// Query filters for logs
#[derive(Debug, Clone, Default, Deserialize)]
pub struct QueryFilters {
    #[serde(default)]
    pub levels: Option<Vec<String>>,
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
}

/// Command to query logs
#[derive(Debug, Clone)]
pub struct QueryLogsCommand {
    pub project_id: String,
    pub filters: QueryFilters,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort: Option<String>,
    pub requesting_user_id: String,
}

// ==================== Responses ====================

/// Response after ingesting logs
#[derive(Debug, Clone, Serialize)]
pub struct IngestResponse {
    pub accepted: u32,
    pub rejected: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

/// Single log entry response
#[derive(Debug, Clone, Serialize)]
pub struct LogResponse {
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

/// Response for log query
#[derive(Debug, Clone, Serialize)]
pub struct LogQueryResponse {
    pub logs: Vec<LogResponse>,
    pub total: i64,
    pub has_more: bool,
}

/// Log level count
#[derive(Debug, Clone, Serialize)]
pub struct LevelCount {
    pub level: String,
    pub count: i64,
}

/// Log statistics response
#[derive(Debug, Clone, Serialize)]
pub struct LogStatsResponse {
    pub total_count: i64,
    pub counts_by_level: Vec<LevelCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_log: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_log: Option<DateTime<Utc>>,
}
