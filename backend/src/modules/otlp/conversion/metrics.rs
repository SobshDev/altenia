//! Convert OTLP metrics to internal metric format

use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

use crate::modules::metrics::application::dto::MetricInput;
use crate::modules::otlp::types::metrics::{ExportMetricsServiceRequest, Metric};

/// Convert OTLP metrics request to internal MetricInput format
pub fn convert_otlp_metrics(request: ExportMetricsServiceRequest) -> Vec<MetricInput> {
    let mut metrics = Vec::new();

    for resource_metrics in request.resource_metrics {
        let resource = resource_metrics.resource.as_ref();

        for scope_metrics in resource_metrics.scope_metrics {
            for metric in scope_metrics.metrics {
                convert_metric(&metric, resource, &mut metrics);
            }
        }
    }

    metrics
}

fn convert_metric(
    metric: &Metric,
    resource: Option<&crate::modules::otlp::types::common::Resource>,
    output: &mut Vec<MetricInput>,
) {
    // Handle Gauge
    if let Some(ref gauge) = metric.gauge {
        for dp in &gauge.data_points {
            let timestamp = parse_nano_timestamp(&dp.time_unix_nano);
            let tags = extract_tags(&dp.attributes, resource);

            output.push(MetricInput {
                name: metric.name.clone(),
                metric_type: "gauge".to_string(),
                value: dp.value(),
                timestamp,
                unit: if metric.unit.is_empty() {
                    None
                } else {
                    Some(metric.unit.clone())
                },
                description: if metric.description.is_empty() {
                    None
                } else {
                    Some(metric.description.clone())
                },
                tags,
                bucket_bounds: None,
                bucket_counts: None,
                histogram_sum: None,
                histogram_count: None,
                histogram_min: None,
                histogram_max: None,
                trace_id: None,
                span_id: None,
            });
        }
    }

    // Handle Sum (Counter)
    if let Some(ref sum) = metric.sum {
        let metric_type = if sum.is_monotonic {
            "counter"
        } else {
            "gauge"
        };

        for dp in &sum.data_points {
            let timestamp = parse_nano_timestamp(&dp.time_unix_nano);
            let tags = extract_tags(&dp.attributes, resource);

            // Extract trace/span from exemplars if available
            let (trace_id, span_id) = dp
                .exemplars
                .first()
                .map(|e| {
                    (
                        if e.trace_id.is_empty() {
                            None
                        } else {
                            Some(e.trace_id.clone())
                        },
                        if e.span_id.is_empty() {
                            None
                        } else {
                            Some(e.span_id.clone())
                        },
                    )
                })
                .unwrap_or((None, None));

            output.push(MetricInput {
                name: metric.name.clone(),
                metric_type: metric_type.to_string(),
                value: dp.value(),
                timestamp,
                unit: if metric.unit.is_empty() {
                    None
                } else {
                    Some(metric.unit.clone())
                },
                description: if metric.description.is_empty() {
                    None
                } else {
                    Some(metric.description.clone())
                },
                tags,
                bucket_bounds: None,
                bucket_counts: None,
                histogram_sum: None,
                histogram_count: None,
                histogram_min: None,
                histogram_max: None,
                trace_id,
                span_id,
            });
        }
    }

    // Handle Histogram
    if let Some(ref histogram) = metric.histogram {
        for dp in &histogram.data_points {
            let timestamp = parse_nano_timestamp(&dp.time_unix_nano);
            let tags = extract_tags(&dp.attributes, resource);

            let bucket_counts: Vec<i64> = dp
                .bucket_counts
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();

            let count: i64 = dp.count.parse().unwrap_or(0);

            // Use sum/count as the value if available
            let value = dp.sum.unwrap_or(0.0);

            output.push(MetricInput {
                name: metric.name.clone(),
                metric_type: "histogram".to_string(),
                value,
                timestamp,
                unit: if metric.unit.is_empty() {
                    None
                } else {
                    Some(metric.unit.clone())
                },
                description: if metric.description.is_empty() {
                    None
                } else {
                    Some(metric.description.clone())
                },
                tags,
                bucket_bounds: Some(dp.explicit_bounds.clone()),
                bucket_counts: Some(bucket_counts),
                histogram_sum: dp.sum,
                histogram_count: Some(count),
                histogram_min: dp.min,
                histogram_max: dp.max,
                trace_id: None,
                span_id: None,
            });
        }
    }

    // Handle Summary (convert to histogram-like)
    if let Some(ref summary) = metric.summary {
        for dp in &summary.data_points {
            let timestamp = parse_nano_timestamp(&dp.time_unix_nano);
            let tags = extract_tags(&dp.attributes, resource);

            let count: i64 = dp.count.parse().unwrap_or(0);

            output.push(MetricInput {
                name: metric.name.clone(),
                metric_type: "histogram".to_string(),
                value: dp.sum,
                timestamp,
                unit: if metric.unit.is_empty() {
                    None
                } else {
                    Some(metric.unit.clone())
                },
                description: if metric.description.is_empty() {
                    None
                } else {
                    Some(metric.description.clone())
                },
                tags,
                bucket_bounds: Some(dp.quantile_values.iter().map(|q| q.quantile).collect()),
                bucket_counts: Some(dp.quantile_values.iter().map(|_| 0).collect()),
                histogram_sum: Some(dp.sum),
                histogram_count: Some(count),
                histogram_min: None,
                histogram_max: None,
                trace_id: None,
                span_id: None,
            });
        }
    }
}

fn extract_tags(
    attrs: &[crate::modules::otlp::types::common::KeyValue],
    resource: Option<&crate::modules::otlp::types::common::Resource>,
) -> HashMap<String, String> {
    let mut tags = HashMap::new();

    // Add resource attributes with "resource." prefix
    if let Some(res) = resource {
        for attr in &res.attributes {
            if let Some(ref s) = attr.value.string_value {
                tags.insert(format!("resource.{}", attr.key), s.clone());
            } else {
                let v = attr.value.to_json_value();
                if !v.is_null() {
                    tags.insert(format!("resource.{}", attr.key), v.to_string());
                }
            }
        }
    }

    // Add metric attributes
    for attr in attrs {
        if let Some(ref s) = attr.value.string_value {
            tags.insert(attr.key.clone(), s.clone());
        } else {
            let v = attr.value.to_json_value();
            if !v.is_null() {
                tags.insert(attr.key.clone(), v.to_string());
            }
        }
    }

    tags
}

/// Parse nanosecond timestamp string to DateTime
fn parse_nano_timestamp(nano_str: &str) -> Option<DateTime<Utc>> {
    if nano_str.is_empty() {
        return None;
    }

    nano_str.parse::<i64>().ok().and_then(|nanos| {
        let secs = nanos / 1_000_000_000;
        let nsecs = (nanos % 1_000_000_000) as u32;
        Utc.timestamp_opt(secs, nsecs).single()
    })
}
