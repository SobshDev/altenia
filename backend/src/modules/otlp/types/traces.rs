//! OTLP Traces types

use serde::{Deserialize, Serialize};

use super::common::{InstrumentationScope, KeyValue, Resource};

/// Export traces request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTraceServiceRequest {
    #[serde(default)]
    pub resource_spans: Vec<ResourceSpans>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpans {
    pub resource: Option<Resource>,
    #[serde(default)]
    pub scope_spans: Vec<ScopeSpans>,
    #[serde(default)]
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeSpans {
    pub scope: Option<InstrumentationScope>,
    #[serde(default)]
    pub spans: Vec<OtlpSpan>,
    #[serde(default)]
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtlpSpan {
    /// Trace ID (hex string, 32 chars)
    pub trace_id: String,
    /// Span ID (hex string, 16 chars)
    pub span_id: String,
    /// Trace state (W3C format)
    #[serde(default)]
    pub trace_state: String,
    /// Parent span ID (hex string, 16 chars)
    #[serde(default)]
    pub parent_span_id: String,
    /// Span name
    pub name: String,
    /// Span kind: 0=unspecified, 1=internal, 2=server, 3=client, 4=producer, 5=consumer
    #[serde(default)]
    pub kind: i32,
    /// Start time in nanoseconds since epoch
    pub start_time_unix_nano: String,
    /// End time in nanoseconds since epoch
    #[serde(default)]
    pub end_time_unix_nano: String,
    /// Span attributes
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub dropped_attributes_count: u32,
    /// Span events
    #[serde(default)]
    pub events: Vec<SpanEvent>,
    #[serde(default)]
    pub dropped_events_count: u32,
    /// Span links
    #[serde(default)]
    pub links: Vec<SpanLink>,
    #[serde(default)]
    pub dropped_links_count: u32,
    /// Status
    pub status: Option<SpanStatus>,
}

impl OtlpSpan {
    /// Convert span kind number to string
    pub fn kind_to_string(&self) -> &'static str {
        match self.kind {
            1 => "internal",
            2 => "server",
            3 => "client",
            4 => "producer",
            5 => "consumer",
            _ => "internal",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanEvent {
    /// Time in nanoseconds since epoch
    pub time_unix_nano: String,
    /// Event name
    pub name: String,
    /// Event attributes
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub dropped_attributes_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanLink {
    /// Trace ID (hex string)
    pub trace_id: String,
    /// Span ID (hex string)
    pub span_id: String,
    /// Trace state
    #[serde(default)]
    pub trace_state: String,
    /// Link attributes
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub dropped_attributes_count: u32,
    #[serde(default)]
    pub flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanStatus {
    /// Status code: 0=unset, 1=ok, 2=error
    #[serde(default)]
    pub code: i32,
    /// Status message
    #[serde(default)]
    pub message: String,
}

impl SpanStatus {
    pub fn code_to_string(&self) -> &'static str {
        match self.code {
            1 => "ok",
            2 => "error",
            _ => "unset",
        }
    }
}

/// Export traces response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTraceServiceResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_success: Option<ExportTracePartialSuccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTracePartialSuccess {
    pub rejected_spans: i64,
    #[serde(default)]
    pub error_message: String,
}
