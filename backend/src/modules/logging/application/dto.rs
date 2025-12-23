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
    /// Metadata field filters (JSON array of {key, operator, value})
    #[serde(default)]
    pub metadata_filters: Option<Vec<MetadataFilterInput>>,
    /// Apply a saved filter preset by ID
    #[serde(default)]
    pub preset_id: Option<String>,
}

/// Metadata filter input for query (simplified version for query params)
#[derive(Debug, Clone, Deserialize)]
pub struct MetadataFilterInput {
    pub key: String,
    pub operator: String,
    #[serde(default)]
    pub value: Option<Value>,
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

// ==================== Filter Preset DTOs ====================

/// Metadata filter DTO for HTTP layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilterDto {
    /// The metadata key to filter on (e.g., "user_id", "request.path")
    pub key: String,
    /// The comparison operator: eq, neq, contains, exists, gt, lt, gte, lte
    pub operator: String,
    /// The value to compare against (None for exists operator)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// Filter configuration DTO for HTTP layer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterConfigDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata_filters: Vec<MetadataFilterDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

/// Command to create a filter preset
#[derive(Debug, Clone)]
pub struct CreateFilterPresetCommand {
    pub project_id: String,
    pub name: String,
    pub filter_config: FilterConfigDto,
    pub is_default: bool,
    pub requesting_user_id: String,
}

/// Command to update a filter preset
#[derive(Debug, Clone)]
pub struct UpdateFilterPresetCommand {
    pub preset_id: String,
    pub project_id: String,
    pub name: Option<String>,
    pub filter_config: Option<FilterConfigDto>,
    pub is_default: Option<bool>,
    pub requesting_user_id: String,
}

/// Command to delete a filter preset
#[derive(Debug, Clone)]
pub struct DeleteFilterPresetCommand {
    pub preset_id: String,
    pub project_id: String,
    pub requesting_user_id: String,
}

/// Command to get a filter preset
#[derive(Debug, Clone)]
pub struct GetFilterPresetCommand {
    pub preset_id: String,
    pub project_id: String,
    pub requesting_user_id: String,
}

/// Command to list filter presets
#[derive(Debug, Clone)]
pub struct ListFilterPresetsCommand {
    pub project_id: String,
    pub requesting_user_id: String,
}

/// Filter preset response
#[derive(Debug, Clone, Serialize)]
pub struct FilterPresetResponse {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub filter_config: FilterConfigDto,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==================== Metrics DTOs ====================

/// Time bucket granularity for metrics
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeBucket {
    Minute,
    #[default]
    Hour,
    Day,
}

impl TimeBucket {
    /// Convert to PostgreSQL interval string
    pub fn to_interval(&self) -> &'static str {
        match self {
            TimeBucket::Minute => "1 minute",
            TimeBucket::Hour => "1 hour",
            TimeBucket::Day => "1 day",
        }
    }
}

/// Command to query metrics
#[derive(Debug, Clone)]
pub struct MetricsQuery {
    pub project_id: String,
    pub bucket: TimeBucket,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub top_sources_limit: i32,
    pub requesting_user_id: String,
}

/// Single time bucket data point
#[derive(Debug, Clone, Serialize)]
pub struct TimeBucketCount {
    pub bucket: DateTime<Utc>,
    pub count: i64,
}

/// Error rate data point (percentage)
#[derive(Debug, Clone, Serialize)]
pub struct ErrorRatePoint {
    pub bucket: DateTime<Utc>,
    pub rate: f64,
}

/// Level-specific time series
#[derive(Debug, Clone, Serialize)]
pub struct LevelTimeSeries {
    pub level: String,
    pub data: Vec<TimeBucketCount>,
}

/// Top source with counts
#[derive(Debug, Clone, Serialize)]
pub struct SourceCount {
    pub source: String,
    pub count: i64,
    pub error_count: i64,
}

/// Full metrics response
#[derive(Debug, Clone, Serialize)]
pub struct MetricsResponse {
    pub volume_over_time: Vec<TimeBucketCount>,
    pub levels_over_time: Vec<LevelTimeSeries>,
    pub error_rate_over_time: Vec<ErrorRatePoint>,
    pub top_sources: Vec<SourceCount>,
    pub summary: LogStatsResponse,
}

// ==================== Export DTOs ====================

/// Request to export logs
#[derive(Debug, Clone, Deserialize)]
pub struct ExportLogsRequest {
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
    /// Maximum number of logs to export (default: 100,000)
    #[serde(default)]
    pub max_logs: Option<i64>,
}

/// Metadata included in the export ZIP
#[derive(Debug, Clone, Serialize)]
pub struct ExportMetadata {
    pub project_id: String,
    pub project_name: String,
    pub exported_at: DateTime<Utc>,
    pub total_logs: i64,
    pub filters: ExportFiltersMetadata,
}

/// Filters used for export (for metadata.json)
#[derive(Debug, Clone, Serialize)]
pub struct ExportFiltersMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}
