use chrono::{DateTime, Utc};
use serde_json::Value;

use super::value_objects::{SpanEvent, SpanKind, SpanLink, SpanStatusCode};
use crate::modules::projects::domain::ProjectId;

/// Span - a single unit of work within a distributed trace
#[derive(Debug, Clone)]
pub struct Span {
    id: String,
    project_id: ProjectId,
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    name: String,
    kind: SpanKind,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    duration_ns: Option<i64>,
    status: SpanStatusCode,
    status_message: Option<String>,
    received_at: DateTime<Utc>,
    service_name: Option<String>,
    service_version: Option<String>,
    resource_attributes: Value,
    attributes: Value,
    events: Vec<SpanEvent>,
    links: Vec<SpanLink>,
}

impl Span {
    /// Create a new span
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        project_id: ProjectId,
        trace_id: String,
        span_id: String,
        parent_span_id: Option<String>,
        name: String,
        kind: SpanKind,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        status: SpanStatusCode,
        status_message: Option<String>,
        service_name: Option<String>,
        service_version: Option<String>,
        resource_attributes: Value,
        attributes: Value,
        events: Vec<SpanEvent>,
        links: Vec<SpanLink>,
    ) -> Self {
        // Calculate duration if end_time is provided
        let duration_ns = end_time.map(|end| {
            (end - start_time).num_nanoseconds().unwrap_or(0)
        });

        Self {
            id,
            project_id,
            trace_id,
            span_id,
            parent_span_id,
            name,
            kind,
            start_time,
            end_time,
            duration_ns,
            status,
            status_message,
            received_at: Utc::now(),
            service_name,
            service_version,
            resource_attributes,
            attributes,
            events,
            links,
        }
    }

    /// Reconstruct from persistence layer
    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: String,
        project_id: ProjectId,
        trace_id: String,
        span_id: String,
        parent_span_id: Option<String>,
        name: String,
        kind: SpanKind,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        duration_ns: Option<i64>,
        status: SpanStatusCode,
        status_message: Option<String>,
        received_at: DateTime<Utc>,
        service_name: Option<String>,
        service_version: Option<String>,
        resource_attributes: Value,
        attributes: Value,
        events: Vec<SpanEvent>,
        links: Vec<SpanLink>,
    ) -> Self {
        Self {
            id,
            project_id,
            trace_id,
            span_id,
            parent_span_id,
            name,
            kind,
            start_time,
            end_time,
            duration_ns,
            status,
            status_message,
            received_at,
            service_name,
            service_version,
            resource_attributes,
            attributes,
            events,
            links,
        }
    }

    // Getters
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    pub fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> SpanKind {
        self.kind
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        self.end_time
    }

    pub fn duration_ns(&self) -> Option<i64> {
        self.duration_ns
    }

    pub fn status(&self) -> SpanStatusCode {
        self.status
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn received_at(&self) -> DateTime<Utc> {
        self.received_at
    }

    pub fn service_name(&self) -> Option<&str> {
        self.service_name.as_deref()
    }

    pub fn service_version(&self) -> Option<&str> {
        self.service_version.as_deref()
    }

    pub fn resource_attributes(&self) -> &Value {
        &self.resource_attributes
    }

    pub fn attributes(&self) -> &Value {
        &self.attributes
    }

    pub fn events(&self) -> &[SpanEvent] {
        &self.events
    }

    pub fn links(&self) -> &[SpanLink] {
        &self.links
    }

    /// Check if this is a root span
    pub fn is_root(&self) -> bool {
        self.parent_span_id.is_none()
    }

    /// Check if this span has an error status
    pub fn has_error(&self) -> bool {
        self.status == SpanStatusCode::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_span() {
        let span = Span::new(
            "span-1".to_string(),
            ProjectId::new("project-1".to_string()),
            "trace-abc123".to_string(),
            "span-xyz789".to_string(),
            None,
            "HTTP GET /api/users".to_string(),
            SpanKind::Server,
            Utc::now(),
            None,
            SpanStatusCode::Unset,
            None,
            Some("user-service".to_string()),
            Some("1.0.0".to_string()),
            json!({}),
            json!({"http.method": "GET"}),
            vec![],
            vec![],
        );

        assert_eq!(span.name(), "HTTP GET /api/users");
        assert_eq!(span.kind(), SpanKind::Server);
        assert!(span.is_root());
        assert!(!span.has_error());
    }

    #[test]
    fn test_span_with_parent() {
        let span = Span::new(
            "span-2".to_string(),
            ProjectId::new("project-1".to_string()),
            "trace-abc123".to_string(),
            "span-child456".to_string(),
            Some("span-parent123".to_string()),
            "Database Query".to_string(),
            SpanKind::Client,
            Utc::now(),
            None,
            SpanStatusCode::Ok,
            None,
            Some("user-service".to_string()),
            None,
            json!({}),
            json!({}),
            vec![],
            vec![],
        );

        assert!(!span.is_root());
        assert_eq!(span.parent_span_id(), Some("span-parent123"));
    }
}
