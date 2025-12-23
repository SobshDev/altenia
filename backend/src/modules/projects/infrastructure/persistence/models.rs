use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database row for projects table
#[derive(Debug, FromRow)]
pub struct ProjectRow {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Database row for api_keys table
#[derive(Debug, FromRow)]
pub struct ApiKeyRow {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
