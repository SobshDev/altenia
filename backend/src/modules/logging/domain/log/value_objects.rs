use serde::{Deserialize, Serialize};

use crate::modules::logging::domain::errors::LogDomainError;

/// Log ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogId(String);

impl LogId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for LogId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Log Level - defines severity of log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Result<Self, LogDomainError> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" | "err" => Ok(Self::Error),
            "fatal" | "critical" => Ok(Self::Fatal),
            _ => Err(LogDomainError::InvalidLevel(format!(
                "unknown level: {}",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }

    /// Numeric severity (higher = more severe)
    pub fn severity(&self) -> u8 {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
            Self::Fatal => 5,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Trace ID - for distributed tracing correlation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceId(String);

impl TraceId {
    pub fn new(id: String) -> Option<Self> {
        if id.is_empty() || id.len() > 64 {
            None
        } else {
            Some(Self(id))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Span ID - for distributed tracing span identification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpanId(String);

impl SpanId {
    pub fn new(id: String) -> Option<Self> {
        if id.is_empty() || id.len() > 64 {
            None
        } else {
            Some(Self(id))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("Info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("warning").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("err").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("fatal").unwrap(), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("critical").unwrap(), LogLevel::Fatal);
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_level_severity() {
        assert!(LogLevel::Fatal.severity() > LogLevel::Error.severity());
        assert!(LogLevel::Error.severity() > LogLevel::Warn.severity());
        assert!(LogLevel::Warn.severity() > LogLevel::Info.severity());
        assert!(LogLevel::Info.severity() > LogLevel::Debug.severity());
        assert!(LogLevel::Debug.severity() > LogLevel::Trace.severity());
    }

    #[test]
    fn test_trace_id() {
        assert!(TraceId::new("abc123".to_string()).is_some());
        assert!(TraceId::new("".to_string()).is_none());
        assert!(TraceId::new("a".repeat(65)).is_none());
    }
}
