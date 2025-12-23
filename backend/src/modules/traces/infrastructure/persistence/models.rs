use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

/// Database row for spans table
#[derive(Debug, Clone, FromRow)]
pub struct SpanRow {
    pub id: String,
    pub project_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ns: Option<i64>,
    pub status: String,
    pub status_message: Option<String>,
    pub received_at: DateTime<Utc>,
    pub service_name: Option<String>,
    pub service_version: Option<String>,
    pub resource_attributes: Value,
    pub attributes: Value,
    pub events: Value,
    pub links: Value,
}

/// Row for trace summary queries
#[derive(Debug, Clone, FromRow)]
pub struct TraceSummaryRow {
    pub trace_id: String,
    pub root_span_name: Option<String>,
    pub service_names: Vec<String>,
    pub span_count: i64,
    pub error_count: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ns: Option<i64>,
}
