use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== Ingest Commands ====================

/// Single metric input for ingestion
#[derive(Debug, Clone, Deserialize)]
pub struct MetricInput {
    pub name: String,
    #[serde(rename = "type")]
    pub metric_type: String,
    pub value: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub unit: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
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

/// Command to ingest metrics
#[derive(Debug, Clone)]
pub struct IngestMetricsCommand {
    pub project_id: String,
    pub metrics: Vec<MetricInput>,
}

// ==================== Query Commands ====================

/// Filters for querying metrics
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetricQueryFilters {
    pub names: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub tags: Option<HashMap<String, String>>,
    pub trace_id: Option<String>,
    pub rollup: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Command to query metrics
#[derive(Debug, Clone)]
pub struct QueryMetricsCommand {
    pub project_id: String,
    pub filters: MetricQueryFilters,
    pub requesting_user_id: String,
}

/// Command to list metric names
#[derive(Debug, Clone)]
pub struct ListMetricNamesCommand {
    pub project_id: String,
    pub requesting_user_id: String,
}

// ==================== Responses ====================

/// Response for ingested metrics
#[derive(Debug, Clone, Serialize)]
pub struct IngestMetricsResponse {
    pub ingested: u32,
}

/// Single aggregated metric data point
#[derive(Debug, Clone, Serialize)]
pub struct MetricDataPoint {
    pub name: String,
    pub metric_type: String,
    pub timestamp: DateTime<Utc>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub sum_value: f64,
    pub sample_count: i64,
}

/// Response for metric queries
#[derive(Debug, Clone, Serialize)]
pub struct MetricQueryResponse {
    pub data: Vec<MetricDataPoint>,
    pub total: i64,
}

/// Response for metric names list
#[derive(Debug, Clone, Serialize)]
pub struct MetricNamesResponse {
    pub names: Vec<String>,
}
