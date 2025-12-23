use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::entity::MetricPoint;
use super::value_objects::MetricType;
use crate::modules::metrics::domain::errors::MetricsDomainError;
use crate::modules::projects::domain::ProjectId;

/// Metric query filters
#[derive(Debug, Clone, Default)]
pub struct MetricFilters {
    pub names: Option<Vec<String>>,
    pub metric_types: Option<Vec<MetricType>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub tags: Option<Vec<(String, String)>>,
    pub trace_id: Option<String>,
}

/// Rollup interval for aggregated queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RollupInterval {
    #[default]
    Raw,
    OneMinute,
    OneHour,
    OneDay,
}

impl RollupInterval {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "raw" | "none" => Self::Raw,
            "1m" | "1min" | "minute" => Self::OneMinute,
            "1h" | "1hour" | "hour" => Self::OneHour,
            "1d" | "1day" | "day" => Self::OneDay,
            _ => Self::Raw,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::OneMinute => "1m",
            Self::OneHour => "1h",
            Self::OneDay => "1d",
        }
    }
}

/// Aggregated metric data point
#[derive(Debug, Clone)]
pub struct AggregatedMetric {
    pub project_id: String,
    pub name: String,
    pub metric_type: String,
    pub bucket: DateTime<Utc>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub sum_value: f64,
    pub sample_count: i64,
}

/// Query result for metrics
#[derive(Debug, Clone)]
pub struct MetricQueryResult {
    pub metrics: Vec<AggregatedMetric>,
    pub total: i64,
}

/// Repository trait for metrics persistence
#[async_trait]
pub trait MetricsRepository: Send + Sync {
    /// Save a batch of metrics
    async fn save_batch(&self, metrics: &[MetricPoint]) -> Result<u32, MetricsDomainError>;

    /// Query metrics with optional aggregation
    async fn query(
        &self,
        project_id: &ProjectId,
        filters: &MetricFilters,
        rollup: RollupInterval,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<MetricQueryResult, MetricsDomainError>;

    /// Get distinct metric names for a project
    async fn get_metric_names(&self, project_id: &ProjectId) -> Result<Vec<String>, MetricsDomainError>;

    /// Delete metrics older than a given timestamp
    async fn delete_before(
        &self,
        project_id: &ProjectId,
        before: DateTime<Utc>,
    ) -> Result<u64, MetricsDomainError>;
}
