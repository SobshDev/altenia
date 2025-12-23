use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database row for organizations table
#[derive(Debug, FromRow)]
pub struct OrganizationRow {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Database row for organization_members table
#[derive(Debug, FromRow)]
pub struct OrganizationMemberRow {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joined query result: organization with user's role
#[derive(Debug, FromRow)]
pub struct OrgWithRoleRow {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub created_at: DateTime<Utc>,
    pub role: String,
}

/// Joined query result: member with user email
#[derive(Debug, FromRow)]
pub struct MemberWithEmailRow {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
