use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum TracesDomainError {
    #[error("Invalid span name: {0}")]
    InvalidSpanName(String),

    #[error("Invalid span kind: {0}")]
    InvalidSpanKind(String),

    #[error("Invalid span status: {0}")]
    InvalidSpanStatus(String),

    #[error("Invalid trace ID: {0}")]
    InvalidTraceId(String),

    #[error("Invalid span ID: {0}")]
    InvalidSpanId(String),

    #[error("Too many spans in trace: {0}")]
    TooManySpans(usize),

    #[error("Too many attributes: {0}")]
    TooManyAttributes(usize),

    #[error("Trace not found")]
    TraceNotFound,

    #[error("Project not found")]
    ProjectNotFound,

    #[error("Project deleted")]
    ProjectDeleted,

    #[error("Not a member of the organization")]
    NotOrgMember,

    #[error("Not authorized")]
    NotAuthorized,

    #[error("Internal error: {0}")]
    InternalError(String),
}
