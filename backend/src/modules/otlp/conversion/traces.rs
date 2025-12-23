//! Convert OTLP traces to internal span format

use chrono::{DateTime, TimeZone, Utc};

use crate::modules::otlp::types::common::attributes_to_json;
use crate::modules::otlp::types::traces::ExportTraceServiceRequest;
use crate::modules::traces::application::dto::{SpanEventInput, SpanInput, SpanLinkInput};

/// Convert OTLP traces request to internal SpanInput format
pub fn convert_otlp_traces(request: ExportTraceServiceRequest) -> Vec<SpanInput> {
    let mut spans = Vec::new();

    for resource_spans in request.resource_spans {
        let resource = resource_spans.resource.as_ref();
        let service_name = resource.and_then(|r| r.get_service_name());
        let service_version = resource.and_then(|r| r.get_service_version());
        let resource_attrs = resource.map(|r| r.to_json()).unwrap_or(serde_json::json!({}));

        for scope_spans in resource_spans.scope_spans {
            for otlp_span in scope_spans.spans {
                let start_time = parse_nano_timestamp(&otlp_span.start_time_unix_nano)
                    .unwrap_or_else(Utc::now);

                let end_time = parse_nano_timestamp(&otlp_span.end_time_unix_nano);

                let status = otlp_span
                    .status
                    .as_ref()
                    .map(|s| s.code_to_string().to_string());

                let status_message = otlp_span
                    .status
                    .as_ref()
                    .filter(|s| !s.message.is_empty())
                    .map(|s| s.message.clone());

                let parent_span_id = if otlp_span.parent_span_id.is_empty() {
                    None
                } else {
                    Some(otlp_span.parent_span_id.clone())
                };

                let attributes = attributes_to_json(&otlp_span.attributes);

                let events: Vec<SpanEventInput> = otlp_span
                    .events
                    .iter()
                    .filter_map(|e| {
                        parse_nano_timestamp(&e.time_unix_nano).map(|ts| SpanEventInput {
                            name: e.name.clone(),
                            timestamp: ts,
                            attributes: attributes_to_json(&e.attributes),
                        })
                    })
                    .collect();

                let links: Vec<SpanLinkInput> = otlp_span
                    .links
                    .iter()
                    .map(|l| SpanLinkInput {
                        trace_id: l.trace_id.clone(),
                        span_id: l.span_id.clone(),
                        attributes: attributes_to_json(&l.attributes),
                    })
                    .collect();

                let kind_str = otlp_span.kind_to_string().to_string();

                spans.push(SpanInput {
                    trace_id: otlp_span.trace_id.clone(),
                    span_id: otlp_span.span_id.clone(),
                    parent_span_id,
                    name: otlp_span.name.clone(),
                    kind: Some(kind_str),
                    start_time,
                    end_time,
                    status,
                    status_message,
                    service_name: service_name.clone(),
                    service_version: service_version.clone(),
                    resource_attributes: resource_attrs.clone(),
                    attributes,
                    events,
                    links,
                });
            }
        }
    }

    spans
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
    use crate::modules::otlp::types::traces::{OtlpSpan, ResourceSpans, ScopeSpans, SpanStatus};

    #[test]
    fn test_convert_simple_span() {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
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
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![OtlpSpan {
                        trace_id: "abc123".to_string(),
                        span_id: "def456".to_string(),
                        trace_state: String::new(),
                        parent_span_id: String::new(),
                        name: "test-operation".to_string(),
                        kind: 2, // server
                        start_time_unix_nano: "1704067200000000000".to_string(),
                        end_time_unix_nano: "1704067201000000000".to_string(),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        events: vec![],
                        dropped_events_count: 0,
                        links: vec![],
                        dropped_links_count: 0,
                        status: Some(SpanStatus {
                            code: 1,
                            message: String::new(),
                        }),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let spans = convert_otlp_traces(request);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].trace_id, "abc123");
        assert_eq!(spans[0].span_id, "def456");
        assert_eq!(spans[0].name, "test-operation");
        assert_eq!(spans[0].kind, Some("server".to_string()));
        assert_eq!(spans[0].status, Some("ok".to_string()));
        assert_eq!(spans[0].service_name, Some("test-service".to_string()));
    }
}
