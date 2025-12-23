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

/// Database row for filter_presets table
#[derive(Debug, FromRow)]
pub struct FilterPresetRow {
    pub id: String,
    pub project_id: String,
    pub user_id: String,
    pub name: String,
    pub filter_config: Value,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Metrics Row Types ====================

/// Row for time bucket counts (volume over time)
#[derive(Debug, FromRow)]
pub struct TimeBucketRow {
    pub bucket: DateTime<Utc>,
    pub count: i64,
}

/// Row for level time series (level + bucket + count)
#[derive(Debug, FromRow)]
pub struct LevelBucketRow {
    pub level: String,
    pub bucket: DateTime<Utc>,
    pub count: i64,
}

/// Row for top sources with error counts
#[derive(Debug, FromRow)]
pub struct SourceCountRow {
    pub source: String,
    pub total: i64,
    pub error_count: i64,
}
