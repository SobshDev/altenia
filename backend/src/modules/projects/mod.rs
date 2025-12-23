pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use application::{ProjectService, ProjectResponse, ApiKeyResponse, ApiKeyCreatedResponse};
pub use domain::{
    ApiKey, ApiKeyId, ApiKeyName, ApiKeyPrefix, ApiKeyRepository,
    Project, ProjectDomainError, ProjectId, ProjectName, ProjectRepository, RetentionDays,
};
pub use infrastructure::{project_routes, PostgresApiKeyRepository, PostgresProjectRepository};
