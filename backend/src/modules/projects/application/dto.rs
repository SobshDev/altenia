use chrono::{DateTime, Utc};

// ==================== Commands ====================

/// Command to create a new project
#[derive(Debug, Clone)]
pub struct CreateProjectCommand {
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub retention_days: Option<i32>,
    pub requesting_user_id: String,
}

/// Command to update a project
#[derive(Debug, Clone)]
pub struct UpdateProjectCommand {
    pub project_id: String,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub retention_days: Option<i32>,
    pub requesting_user_id: String,
}

/// Command to delete a project
#[derive(Debug, Clone)]
pub struct DeleteProjectCommand {
    pub project_id: String,
    pub requesting_user_id: String,
}

/// Command to create an API key
#[derive(Debug, Clone)]
pub struct CreateApiKeyCommand {
    pub project_id: String,
    pub name: String,
    pub expires_in_days: Option<i64>,
    pub requesting_user_id: String,
}

/// Command to revoke an API key
#[derive(Debug, Clone)]
pub struct RevokeApiKeyCommand {
    pub project_id: String,
    pub api_key_id: String,
    pub requesting_user_id: String,
}

// ==================== Responses ====================

/// Response for project data
#[derive(Debug, Clone)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: String,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for API key data (without the actual key)
#[derive(Debug, Clone)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Response when creating an API key (includes the plain key - only shown once)
#[derive(Debug, Clone)]
pub struct ApiKeyCreatedResponse {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub plain_key: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
