//! OTLP Metrics types

use serde::{Deserialize, Serialize};

use super::common::{InstrumentationScope, KeyValue, Resource};

/// Export metrics request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetricsServiceRequest {
    #[serde(default)]
    pub resource_metrics: Vec<ResourceMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetrics {
    pub resource: Option<Resource>,
    #[serde(default)]
    pub scope_metrics: Vec<ScopeMetrics>,
    #[serde(default)]
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeMetrics {
    pub scope: Option<InstrumentationScope>,
    #[serde(default)]
    pub metrics: Vec<Metric>,
    #[serde(default)]
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub unit: String,
    // One of these will be set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gauge: Option<Gauge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<Sum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histogram: Option<Histogram>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exponential_histogram: Option<ExponentialHistogram>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauge {
    #[serde(default)]
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sum {
    #[serde(default)]
    pub data_points: Vec<NumberDataPoint>,
    /// AggregationTemporality: 0=unspecified, 1=delta, 2=cumulative
    #[serde(default)]
    pub aggregation_temporality: i32,
    #[serde(default)]
    pub is_monotonic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberDataPoint {
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    /// Start time in nanoseconds
    #[serde(default)]
    pub start_time_unix_nano: String,
    /// Time in nanoseconds
    #[serde(default)]
    pub time_unix_nano: String,
    // One of these will be set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_double: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_int: Option<String>, // int64 as string
    #[serde(default)]
    pub exemplars: Vec<Exemplar>,
    #[serde(default)]
    pub flags: u32,
}

impl NumberDataPoint {
    pub fn value(&self) -> f64 {
        if let Some(d) = self.as_double {
            d
        } else if let Some(ref i) = self.as_int {
            i.parse().unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Histogram {
    #[serde(default)]
    pub data_points: Vec<HistogramDataPoint>,
    /// AggregationTemporality: 0=unspecified, 1=delta, 2=cumulative
    #[serde(default)]
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistogramDataPoint {
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub start_time_unix_nano: String,
    #[serde(default)]
    pub time_unix_nano: String,
    /// Total count of observations
    #[serde(default)]
    pub count: String, // uint64 as string
    /// Sum of observations (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    /// Bucket counts (length = explicit_bounds.len() + 1)
    #[serde(default)]
    pub bucket_counts: Vec<String>, // uint64 as strings
    /// Bucket boundaries (upper bounds, exclusive)
    #[serde(default)]
    pub explicit_bounds: Vec<f64>,
    #[serde(default)]
    pub exemplars: Vec<Exemplar>,
    #[serde(default)]
    pub flags: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExponentialHistogram {
    #[serde(default)]
    pub data_points: Vec<ExponentialHistogramDataPoint>,
    #[serde(default)]
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExponentialHistogramDataPoint {
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub start_time_unix_nano: String,
    #[serde(default)]
    pub time_unix_nano: String,
    #[serde(default)]
    pub count: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(default)]
    pub scale: i32,
    #[serde(default)]
    pub zero_count: String,
    pub positive: Option<Buckets>,
    pub negative: Option<Buckets>,
    #[serde(default)]
    pub flags: u32,
    #[serde(default)]
    pub exemplars: Vec<Exemplar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buckets {
    #[serde(default)]
    pub offset: i32,
    #[serde(default)]
    pub bucket_counts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    #[serde(default)]
    pub data_points: Vec<SummaryDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDataPoint {
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub start_time_unix_nano: String,
    #[serde(default)]
    pub time_unix_nano: String,
    #[serde(default)]
    pub count: String,
    #[serde(default)]
    pub sum: f64,
    #[serde(default)]
    pub quantile_values: Vec<ValueAtQuantile>,
    #[serde(default)]
    pub flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueAtQuantile {
    pub quantile: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exemplar {
    #[serde(default)]
    pub filtered_attributes: Vec<KeyValue>,
    #[serde(default)]
    pub time_unix_nano: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_double: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_int: Option<String>,
    #[serde(default)]
    pub span_id: String,
    #[serde(default)]
    pub trace_id: String,
}

/// Export metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetricsServiceResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_success: Option<ExportMetricsPartialSuccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetricsPartialSuccess {
    pub rejected_data_points: i64,
    #[serde(default)]
    pub error_message: String,
}
