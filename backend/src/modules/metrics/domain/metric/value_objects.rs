use crate::modules::metrics::domain::errors::MetricsDomainError;

/// Metric Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

impl MetricType {
    pub fn from_str(s: &str) -> Result<Self, MetricsDomainError> {
        match s.to_lowercase().as_str() {
            "counter" => Ok(Self::Counter),
            "gauge" => Ok(Self::Gauge),
            "histogram" => Ok(Self::Histogram),
            _ => Err(MetricsDomainError::InvalidMetricType(format!(
                "Unknown metric type: {}. Must be 'counter', 'gauge', or 'histogram'",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }
}

/// Histogram Data for histogram metrics
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramData {
    bucket_bounds: Vec<f64>,
    bucket_counts: Vec<i64>,
    sum: f64,
    count: i64,
    min: f64,
    max: f64,
}

impl HistogramData {
    pub fn new(
        bucket_bounds: Vec<f64>,
        bucket_counts: Vec<i64>,
        sum: f64,
        count: i64,
        min: f64,
        max: f64,
    ) -> Result<Self, MetricsDomainError> {
        // Validate bucket bounds and counts have compatible lengths
        // bucket_counts should have one more element than bucket_bounds (for the +Inf bucket)
        if bucket_counts.len() != bucket_bounds.len() + 1 && bucket_counts.len() != bucket_bounds.len() {
            return Err(MetricsDomainError::InvalidHistogramData(format!(
                "Bucket counts length ({}) must equal bucket bounds length ({}) or bounds length + 1",
                bucket_counts.len(),
                bucket_bounds.len()
            )));
        }

        // Validate bucket bounds are in ascending order
        for i in 1..bucket_bounds.len() {
            if bucket_bounds[i] <= bucket_bounds[i - 1] {
                return Err(MetricsDomainError::InvalidHistogramData(
                    "Bucket bounds must be in strictly ascending order".to_string(),
                ));
            }
        }

        // Validate count is non-negative
        if count < 0 {
            return Err(MetricsDomainError::InvalidHistogramData(
                "Count must be non-negative".to_string(),
            ));
        }

        Ok(Self {
            bucket_bounds,
            bucket_counts,
            sum,
            count,
            min,
            max,
        })
    }

    pub fn bucket_bounds(&self) -> &[f64] {
        &self.bucket_bounds
    }

    pub fn bucket_counts(&self) -> &[i64] {
        &self.bucket_counts
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn count(&self) -> i64 {
        self.count
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_type_from_str() {
        assert!(matches!(MetricType::from_str("counter"), Ok(MetricType::Counter)));
        assert!(matches!(MetricType::from_str("Counter"), Ok(MetricType::Counter)));
        assert!(matches!(MetricType::from_str("gauge"), Ok(MetricType::Gauge)));
        assert!(matches!(MetricType::from_str("histogram"), Ok(MetricType::Histogram)));
        assert!(MetricType::from_str("invalid").is_err());
    }

    #[test]
    fn test_histogram_data_valid() {
        let bounds = vec![10.0, 50.0, 100.0];
        let counts = vec![5, 10, 3, 2]; // 4 buckets: [0-10), [10-50), [50-100), [100-+Inf)
        let data = HistogramData::new(bounds, counts, 1500.0, 20, 5.0, 150.0);
        assert!(data.is_ok());
    }

    #[test]
    fn test_histogram_data_invalid_bounds_order() {
        let bounds = vec![100.0, 50.0, 10.0]; // Wrong order
        let counts = vec![5, 10, 3, 2];
        let data = HistogramData::new(bounds, counts, 1500.0, 20, 5.0, 150.0);
        assert!(data.is_err());
    }
}
