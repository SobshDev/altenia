use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MetricsDomainError {
    #[error("Invalid metric name: {0}")]
    InvalidMetricName(String),

    #[error("Invalid metric type: {0}")]
    InvalidMetricType(String),

    #[error("Invalid metric value: {0}")]
    InvalidMetricValue(String),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Invalid histogram data: {0}")]
    InvalidHistogramData(String),

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
