use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::entity::Span;
use super::value_objects::SpanStatusCode;
use crate::modules::traces::domain::errors::TracesDomainError;
use crate::modules::projects::domain::ProjectId;

/// Filters for trace queries
#[derive(Debug, Clone, Default)]
pub struct TraceFilters {
    pub service_name: Option<String>,
    pub span_name: Option<String>,
    pub status: Option<SpanStatusCode>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_duration_ns: Option<i64>,
    pub max_duration_ns: Option<i64>,
}

/// Pagination parameters
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

/// Summary of a trace for listing
#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub trace_id: String,
    pub root_span_name: Option<String>,
    pub service_names: Vec<String>,
    pub span_count: i64,
    pub error_count: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ns: Option<i64>,
}

/// Result of a trace search
#[derive(Debug, Clone)]
pub struct TraceSearchResult {
    pub traces: Vec<TraceSummary>,
    pub total: i64,
}

/// Repository trait for spans persistence
#[async_trait]
pub trait SpansRepository: Send + Sync {
    /// Save a batch of spans
    async fn save_batch(&self, spans: &[Span]) -> Result<u32, TracesDomainError>;

    /// Get all spans for a trace
    async fn get_trace(
        &self,
        project_id: &ProjectId,
        trace_id: &str,
    ) -> Result<Vec<Span>, TracesDomainError>;

    /// Search traces
    async fn search_traces(
        &self,
        project_id: &ProjectId,
        filters: &TraceFilters,
        pagination: &Pagination,
    ) -> Result<TraceSearchResult, TracesDomainError>;

    /// Get distinct service names for a project
    async fn get_service_names(&self, project_id: &ProjectId) -> Result<Vec<String>, TracesDomainError>;

    /// Delete spans older than a given timestamp
    async fn delete_before(
        &self,
        project_id: &ProjectId,
        before: DateTime<Utc>,
    ) -> Result<u64, TracesDomainError>;
}
