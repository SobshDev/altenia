use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database row for users table
#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for refresh_tokens table
#[derive(Debug, FromRow)]
pub struct RefreshTokenRow {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub device_fingerprint: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
