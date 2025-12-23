use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LogDomainError {
    // Validation errors
    InvalidLevel(String),
    InvalidTimestamp(String),
    InvalidMessage(String),

    // Project errors
    ProjectNotFound,
    ProjectDeleted,

    // API Key errors
    ApiKeyInvalid,
    ApiKeyRevoked,
    ApiKeyExpired,

    // Permission errors
    InsufficientPermissions,
    NotOrgMember,

    // Infrastructure errors
    InternalError(String),
}

impl fmt::Display for LogDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLevel(msg) => write!(f, "Invalid log level: {}", msg),
            Self::InvalidTimestamp(msg) => write!(f, "Invalid timestamp: {}", msg),
            Self::InvalidMessage(msg) => write!(f, "Invalid message: {}", msg),
            Self::ProjectNotFound => write!(f, "Project not found"),
            Self::ProjectDeleted => write!(f, "Project has been deleted"),
            Self::ApiKeyInvalid => write!(f, "Invalid API key"),
            Self::ApiKeyRevoked => write!(f, "API key has been revoked"),
            Self::ApiKeyExpired => write!(f, "API key has expired"),
            Self::InsufficientPermissions => write!(f, "Insufficient permissions for this action"),
            Self::NotOrgMember => write!(f, "User is not a member of this organization"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for LogDomainError {}
