use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::value_objects::{HistogramData, MetricType};
use crate::modules::projects::domain::ProjectId;

/// MetricPoint - a single metric data point
#[derive(Debug, Clone)]
pub struct MetricPoint {
    id: String,
    project_id: ProjectId,
    name: String,
    metric_type: MetricType,
    value: f64,
    timestamp: DateTime<Utc>,
    received_at: DateTime<Utc>,
    unit: Option<String>,
    description: Option<String>,
    tags: HashMap<String, String>,
    histogram_data: Option<HistogramData>,
    trace_id: Option<String>,
    span_id: Option<String>,
}

impl MetricPoint {
    /// Create a new counter or gauge metric
    pub fn new(
        id: String,
        project_id: ProjectId,
        name: String,
        metric_type: MetricType,
        value: f64,
        timestamp: DateTime<Utc>,
        unit: Option<String>,
        description: Option<String>,
        tags: HashMap<String, String>,
        trace_id: Option<String>,
        span_id: Option<String>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            metric_type,
            value,
            timestamp,
            received_at: Utc::now(),
            unit,
            description,
            tags,
            histogram_data: None,
            trace_id,
            span_id,
        }
    }

    /// Create a new histogram metric
    pub fn new_histogram(
        id: String,
        project_id: ProjectId,
        name: String,
        value: f64,
        timestamp: DateTime<Utc>,
        unit: Option<String>,
        description: Option<String>,
        tags: HashMap<String, String>,
        histogram_data: HistogramData,
        trace_id: Option<String>,
        span_id: Option<String>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            metric_type: MetricType::Histogram,
            value,
            timestamp,
            received_at: Utc::now(),
            unit,
            description,
            tags,
            histogram_data: Some(histogram_data),
            trace_id,
            span_id,
        }
    }

    /// Reconstruct from persistence layer
    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: String,
        project_id: ProjectId,
        name: String,
        metric_type: MetricType,
        value: f64,
        timestamp: DateTime<Utc>,
        received_at: DateTime<Utc>,
        unit: Option<String>,
        description: Option<String>,
        tags: HashMap<String, String>,
        histogram_data: Option<HistogramData>,
        trace_id: Option<String>,
        span_id: Option<String>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            metric_type,
            value,
            timestamp,
            received_at,
            unit,
            description,
            tags,
            histogram_data,
            trace_id,
            span_id,
        }
    }

    // Getters
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn received_at(&self) -> DateTime<Utc> {
        self.received_at
    }

    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn tags(&self) -> &HashMap<String, String> {
        &self.tags
    }

    pub fn histogram_data(&self) -> Option<&HistogramData> {
        self.histogram_data.as_ref()
    }

    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    pub fn span_id(&self) -> Option<&str> {
        self.span_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_counter_metric() {
        let metric = MetricPoint::new(
            "metric-1".to_string(),
            ProjectId::new("project-1".to_string()),
            "http_requests_total".to_string(),
            MetricType::Counter,
            100.0,
            Utc::now(),
            Some("requests".to_string()),
            None,
            HashMap::new(),
            None,
            None,
        );

        assert_eq!(metric.name(), "http_requests_total");
        assert_eq!(metric.metric_type(), MetricType::Counter);
        assert_eq!(metric.value(), 100.0);
        assert!(metric.histogram_data().is_none());
    }

    #[test]
    fn test_new_histogram_metric() {
        let histogram = HistogramData::new(
            vec![10.0, 50.0, 100.0],
            vec![5, 10, 3, 2],
            1500.0,
            20,
            5.0,
            150.0,
        )
        .unwrap();

        let metric = MetricPoint::new_histogram(
            "metric-2".to_string(),
            ProjectId::new("project-1".to_string()),
            "http_request_duration_seconds".to_string(),
            0.5,
            Utc::now(),
            Some("seconds".to_string()),
            None,
            HashMap::new(),
            histogram,
            None,
            None,
        );

        assert_eq!(metric.metric_type(), MetricType::Histogram);
        assert!(metric.histogram_data().is_some());
    }
}
