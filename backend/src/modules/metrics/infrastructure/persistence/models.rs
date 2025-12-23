use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

/// Database row for metrics table
#[derive(Debug, FromRow)]
pub struct MetricRow {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub metric_type: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub unit: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Value>,
    // Histogram-specific
    pub bucket_bounds: Option<Vec<f64>>,
    pub bucket_counts: Option<Vec<i64>>,
    pub histogram_sum: Option<f64>,
    pub histogram_count: Option<i64>,
    pub histogram_min: Option<f64>,
    pub histogram_max: Option<f64>,
    // Correlation
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Aggregated metric row from continuous aggregates
#[derive(Debug, FromRow)]
pub struct AggregatedMetricRow {
    pub project_id: String,
    pub name: String,
    pub metric_type: String,
    pub bucket: DateTime<Utc>,
    pub avg_value: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub sum_value: Option<f64>,
    pub sample_count: Option<i64>,
}

/// Metric name row
#[derive(Debug, FromRow)]
pub struct MetricNameRow {
    pub name: String,
}
