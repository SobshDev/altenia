use chrono::{DateTime, Utc};
use serde_json::Value;

use super::value_objects::{LogId, LogLevel, SpanId, TraceId};
use crate::modules::projects::domain::ProjectId;

/// LogEntry - represents a single log record
#[derive(Debug, Clone)]
pub struct LogEntry {
    id: LogId,
    project_id: ProjectId,
    level: LogLevel,
    message: String,
    timestamp: DateTime<Utc>,
    received_at: DateTime<Utc>,
    source: Option<String>,
    metadata: Option<Value>,
    trace_id: Option<TraceId>,
    span_id: Option<SpanId>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(
        id: LogId,
        project_id: ProjectId,
        level: LogLevel,
        message: String,
        timestamp: Option<DateTime<Utc>>,
        source: Option<String>,
        metadata: Option<Value>,
        trace_id: Option<TraceId>,
        span_id: Option<SpanId>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            project_id,
            level,
            message,
            timestamp: timestamp.unwrap_or(now),
            received_at: now,
            source,
            metadata,
            trace_id,
            span_id,
        }
    }

    /// Reconstruct from persistence layer
    pub fn reconstruct(
        id: LogId,
        project_id: ProjectId,
        level: LogLevel,
        message: String,
        timestamp: DateTime<Utc>,
        received_at: DateTime<Utc>,
        source: Option<String>,
        metadata: Option<Value>,
        trace_id: Option<TraceId>,
        span_id: Option<SpanId>,
    ) -> Self {
        Self {
            id,
            project_id,
            level,
            message,
            timestamp,
            received_at,
            source,
            metadata,
            trace_id,
            span_id,
        }
    }

    // Getters
    pub fn id(&self) -> &LogId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn received_at(&self) -> DateTime<Utc> {
        self.received_at
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn metadata(&self) -> Option<&Value> {
        self.metadata.as_ref()
    }

    pub fn trace_id(&self) -> Option<&TraceId> {
        self.trace_id.as_ref()
    }

    pub fn span_id(&self) -> Option<&SpanId> {
        self.span_id.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_log_entry() {
        let id = LogId::new("log-123".to_string());
        let project_id = ProjectId::new("proj-456".to_string());
        let entry = LogEntry::new(
            id,
            project_id,
            LogLevel::Info,
            "Test message".to_string(),
            None,
            Some("app.main".to_string()),
            None,
            None,
            None,
        );

        assert_eq!(entry.level(), LogLevel::Info);
        assert_eq!(entry.message(), "Test message");
        assert_eq!(entry.source(), Some("app.main"));
        assert!(entry.metadata().is_none());
    }

    #[test]
    fn test_log_entry_with_metadata() {
        let id = LogId::new("log-123".to_string());
        let project_id = ProjectId::new("proj-456".to_string());
        let metadata = serde_json::json!({
            "user_id": "user-789",
            "request_id": "req-abc"
        });

        let entry = LogEntry::new(
            id,
            project_id,
            LogLevel::Error,
            "Something went wrong".to_string(),
            None,
            None,
            Some(metadata.clone()),
            TraceId::new("trace-123".to_string()),
            SpanId::new("span-456".to_string()),
        );

        assert_eq!(entry.level(), LogLevel::Error);
        assert_eq!(entry.metadata(), Some(&metadata));
        assert!(entry.trace_id().is_some());
        assert!(entry.span_id().is_some());
    }
}
