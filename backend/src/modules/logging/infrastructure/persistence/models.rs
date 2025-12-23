use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

/// Database row for logs table (TimescaleDB hypertable)
#[derive(Debug, FromRow)]
pub struct LogRow {
    pub id: String,
    pub project_id: String,
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub source: Option<String>,
    pub metadata: Option<Value>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Row for level counts
#[derive(Debug, FromRow)]
pub struct LevelCountRow {
    pub level: String,
    pub count: i64,
}

/// Row for log stats
#[derive(Debug, FromRow)]
pub struct LogStatsRow {
    pub total_count: i64,
    pub oldest_log: Option<DateTime<Utc>>,
    pub newest_log: Option<DateTime<Utc>>,
}
