use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectDomainError {
    // Validation errors
    InvalidProjectName(String),
    InvalidRetentionDays(String),
    InvalidApiKeyName(String),

    // Project errors
    ProjectNotFound,
    ProjectAlreadyExists,
    ProjectAlreadyDeleted,

    // API Key errors
    ApiKeyNotFound,
    ApiKeyRevoked,
    ApiKeyExpired,
    ApiKeyInvalid,

    // Permission errors
    InsufficientPermissions,
    NotOrgMember,

    // Infrastructure errors
    InternalError(String),
}

impl fmt::Display for ProjectDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProjectName(msg) => write!(f, "Invalid project name: {}", msg),
            Self::InvalidRetentionDays(msg) => write!(f, "Invalid retention days: {}", msg),
            Self::InvalidApiKeyName(msg) => write!(f, "Invalid API key name: {}", msg),
            Self::ProjectNotFound => write!(f, "Project not found"),
            Self::ProjectAlreadyExists => write!(f, "Project already exists in this organization"),
            Self::ProjectAlreadyDeleted => write!(f, "Project is already deleted"),
            Self::ApiKeyNotFound => write!(f, "API key not found"),
            Self::ApiKeyRevoked => write!(f, "API key has been revoked"),
            Self::ApiKeyExpired => write!(f, "API key has expired"),
            Self::ApiKeyInvalid => write!(f, "API key is invalid"),
            Self::InsufficientPermissions => write!(f, "Insufficient permissions for this action"),
            Self::NotOrgMember => write!(f, "User is not a member of this organization"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProjectDomainError {}
