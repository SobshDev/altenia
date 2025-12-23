//! Convert OTLP logs to internal log format

use chrono::{DateTime, TimeZone, Utc};

use crate::modules::logging::application::dto::LogInput;
use crate::modules::otlp::types::common::attributes_to_json;
use crate::modules::otlp::types::logs::ExportLogsServiceRequest;

/// Convert OTLP logs request to internal LogInput format
pub fn convert_otlp_logs(request: ExportLogsServiceRequest) -> Vec<LogInput> {
    let mut logs = Vec::new();

    for resource_logs in request.resource_logs {
        let resource = resource_logs.resource.as_ref();
        let service_name = resource.and_then(|r| r.get_service_name());
        let resource_attrs = resource.map(|r| r.to_json());

        for scope_logs in resource_logs.scope_logs {
            for record in scope_logs.log_records {
                let timestamp = parse_nano_timestamp(&record.time_unix_nano)
                    .or_else(|| parse_nano_timestamp(&record.observed_time_unix_nano));

                let message = record
                    .body
                    .as_ref()
                    .map(|v| v.to_json_value())
                    .map(|v| match v {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    })
                    .unwrap_or_default();

                let level = record.severity_to_level();

                // Combine resource attributes with log attributes
                let mut metadata = serde_json::Map::new();
                if let Some(ref res_attrs) = resource_attrs {
                    if let serde_json::Value::Object(m) = res_attrs {
                        for (k, v) in m {
                            metadata.insert(format!("resource.{}", k), v.clone());
                        }
                    }
                }
                let log_attrs = attributes_to_json(&record.attributes);
                if let serde_json::Value::Object(m) = log_attrs {
                    for (k, v) in m {
                        metadata.insert(k, v);
                    }
                }

                let trace_id = if record.trace_id.is_empty() {
                    None
                } else {
                    Some(record.trace_id.clone())
                };

                let span_id = if record.span_id.is_empty() {
                    None
                } else {
                    Some(record.span_id.clone())
                };

                logs.push(LogInput {
                    level,
                    message,
                    timestamp,
                    source: service_name.clone(),
                    metadata: if metadata.is_empty() {
                        None
                    } else {
                        Some(serde_json::Value::Object(metadata))
                    },
                    trace_id,
                    span_id,
                });
            }
        }
    }

    logs
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::otlp::types::common::{AnyValue, KeyValue, Resource};
    use crate::modules::otlp::types::logs::{LogRecord, ResourceLogs, ScopeLogs};

    #[test]
    fn test_convert_simple_log() {
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: AnyValue {
                            string_value: Some("test-service".to_string()),
                            ..Default::default()
                        },
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: "1704067200000000000".to_string(), // 2024-01-01 00:00:00
                        observed_time_unix_nano: String::new(),
                        severity_number: 9,
                        severity_text: String::new(),
                        body: Some(AnyValue {
                            string_value: Some("Test log message".to_string()),
                            ..Default::default()
                        }),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        flags: 0,
                        trace_id: String::new(),
                        span_id: String::new(),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let logs = convert_otlp_logs(request);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "INFO");
        assert_eq!(logs[0].message, "Test log message");
        assert_eq!(logs[0].source, Some("test-service".to_string()));
    }
}

