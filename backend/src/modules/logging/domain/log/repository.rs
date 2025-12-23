use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::entity::LogEntry;
use super::value_objects::LogLevel;
use crate::modules::logging::domain::errors::LogDomainError;
use crate::modules::logging::domain::filter_preset::MetadataFilter;
use crate::modules::projects::domain::ProjectId;

/// Query filters for logs
#[derive(Debug, Clone, Default)]
pub struct LogFilters {
    pub levels: Option<Vec<LogLevel>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub search: Option<String>,
    pub trace_id: Option<String>,
    /// Metadata field filters (JSONB queries)
    pub metadata_filters: Vec<MetadataFilter>,
}

/// Pagination options
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Descending
    }
}

impl SortOrder {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Self::Ascending,
            _ => Self::Descending,
        }
    }
}

/// Query result with pagination info
#[derive(Debug, Clone)]
pub struct LogQueryResult {
    pub logs: Vec<LogEntry>,
    pub total: i64,
    pub has_more: bool,
}

/// Log statistics
#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_count: i64,
    pub counts_by_level: Vec<(LogLevel, i64)>,
    pub oldest_log: Option<DateTime<Utc>>,
    pub newest_log: Option<DateTime<Utc>>,
}

/// Repository trait for Log persistence
#[async_trait]
pub trait LogRepository: Send + Sync {
    /// Save a batch of log entries
    async fn save_batch(&self, logs: &[LogEntry]) -> Result<u32, LogDomainError>;

    /// Query logs with filters and pagination
    async fn query(
        &self,
        project_id: &ProjectId,
        filters: &LogFilters,
        pagination: &Pagination,
        sort: SortOrder,
    ) -> Result<LogQueryResult, LogDomainError>;

    /// Count logs matching filters
    async fn count(
        &self,
        project_id: &ProjectId,
        filters: &LogFilters,
    ) -> Result<i64, LogDomainError>;

    /// Get log statistics for a project
    async fn get_stats(&self, project_id: &ProjectId) -> Result<LogStats, LogDomainError>;

    /// Delete logs older than a given timestamp (for retention)
    async fn delete_before(
        &self,
        project_id: &ProjectId,
        before: DateTime<Utc>,
    ) -> Result<u64, LogDomainError>;
}
