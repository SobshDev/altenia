use crate::modules::traces::domain::errors::TracesDomainError;

/// Span Kind - describes the relationship between the span and its parent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

impl SpanKind {
    pub fn from_str(s: &str) -> Result<Self, TracesDomainError> {
        match s.to_lowercase().as_str() {
            "internal" => Ok(Self::Internal),
            "server" => Ok(Self::Server),
            "client" => Ok(Self::Client),
            "producer" => Ok(Self::Producer),
            "consumer" => Ok(Self::Consumer),
            _ => Err(TracesDomainError::InvalidSpanKind(format!(
                "Unknown span kind: {}. Must be 'internal', 'server', 'client', 'producer', or 'consumer'",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::Server => "server",
            Self::Client => "client",
            Self::Producer => "producer",
            Self::Consumer => "consumer",
        }
    }
}

impl Default for SpanKind {
    fn default() -> Self {
        Self::Internal
    }
}

/// Span Status Code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatusCode {
    Unset,
    Ok,
    Error,
}

impl SpanStatusCode {
    pub fn from_str(s: &str) -> Result<Self, TracesDomainError> {
        match s.to_lowercase().as_str() {
            "unset" => Ok(Self::Unset),
            "ok" => Ok(Self::Ok),
            "error" => Ok(Self::Error),
            _ => Err(TracesDomainError::InvalidSpanStatus(format!(
                "Unknown span status: {}. Must be 'unset', 'ok', or 'error'",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unset => "unset",
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

impl Default for SpanStatusCode {
    fn default() -> Self {
        Self::Unset
    }
}

/// Span Event - a timestamped event within a span
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub attributes: serde_json::Value,
}

/// Span Link - a reference to another span
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpanLink {
    pub trace_id: String,
    pub span_id: String,
    pub attributes: serde_json::Value,
}

/// Limits for spans
pub const MAX_SPANS_PER_TRACE: usize = 500;
pub const MAX_ATTRIBUTES_PER_SPAN: usize = 64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_kind_from_str() {
        assert!(matches!(SpanKind::from_str("internal"), Ok(SpanKind::Internal)));
        assert!(matches!(SpanKind::from_str("server"), Ok(SpanKind::Server)));
        assert!(matches!(SpanKind::from_str("client"), Ok(SpanKind::Client)));
        assert!(matches!(SpanKind::from_str("producer"), Ok(SpanKind::Producer)));
        assert!(matches!(SpanKind::from_str("consumer"), Ok(SpanKind::Consumer)));
        assert!(SpanKind::from_str("invalid").is_err());
    }

    #[test]
    fn test_span_status_from_str() {
        assert!(matches!(SpanStatusCode::from_str("unset"), Ok(SpanStatusCode::Unset)));
        assert!(matches!(SpanStatusCode::from_str("ok"), Ok(SpanStatusCode::Ok)));
        assert!(matches!(SpanStatusCode::from_str("error"), Ok(SpanStatusCode::Error)));
        assert!(SpanStatusCode::from_str("invalid").is_err());
    }
}
