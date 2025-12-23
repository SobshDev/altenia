//! OTLP Logs types

use serde::{Deserialize, Serialize};

use super::common::{AnyValue, InstrumentationScope, KeyValue, Resource};

/// Export logs request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLogsServiceRequest {
    #[serde(default)]
    pub resource_logs: Vec<ResourceLogs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLogs {
    pub resource: Option<Resource>,
    #[serde(default)]
    pub scope_logs: Vec<ScopeLogs>,
    #[serde(default)]
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeLogs {
    pub scope: Option<InstrumentationScope>,
    #[serde(default)]
    pub log_records: Vec<LogRecord>,
    #[serde(default)]
    pub schema_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRecord {
    /// Timestamp in nanoseconds since epoch (as string for int64)
    #[serde(default)]
    pub time_unix_nano: String,
    /// Observed timestamp in nanoseconds since epoch
    #[serde(default)]
    pub observed_time_unix_nano: String,
    /// Severity number (1-24)
    #[serde(default)]
    pub severity_number: i32,
    /// Severity text (e.g., "INFO", "ERROR")
    #[serde(default)]
    pub severity_text: String,
    /// Log body
    pub body: Option<AnyValue>,
    /// Log attributes
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub dropped_attributes_count: u32,
    /// Flags
    #[serde(default)]
    pub flags: u32,
    /// Trace ID (hex string, 32 chars)
    #[serde(default)]
    pub trace_id: String,
    /// Span ID (hex string, 16 chars)
    #[serde(default)]
    pub span_id: String,
}

impl LogRecord {
    /// Convert severity number to level string
    pub fn severity_to_level(&self) -> String {
        if !self.severity_text.is_empty() {
            return self.severity_text.to_uppercase();
        }

        match self.severity_number {
            1..=4 => "TRACE".to_string(),
            5..=8 => "DEBUG".to_string(),
            9..=12 => "INFO".to_string(),
            13..=16 => "WARN".to_string(),
            17..=20 => "ERROR".to_string(),
            21..=24 => "FATAL".to_string(),
            _ => "INFO".to_string(),
        }
    }
}

/// Export logs response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLogsServiceResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_success: Option<ExportLogsPartialSuccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLogsPartialSuccess {
    pub rejected_log_records: i64,
    #[serde(default)]
    pub error_message: String,
}
