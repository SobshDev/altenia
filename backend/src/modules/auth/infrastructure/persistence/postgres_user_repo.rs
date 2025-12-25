use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::UserRow;
use crate::modules::auth::domain::{
    AuthDomainError, DisplayName, Email, PasswordHash, User, UserId, UserRepository,
};

/// PostgreSQL implementation of UserRepository
pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_user(row: UserRow) -> Result<User, AuthDomainError> {
        let email = Email::new(row.email)?;
        let user_id = UserId::new(row.id);
        let password_hash = row.password_hash.map(PasswordHash::from_hash);
        let display_name = row
            .display_name
            .map(DisplayName::new)
            .transpose()?;

        Ok(User::reconstruct(
            user_id,
            email,
            password_hash,
            display_name,
            row.allow_invites,
            row.created_at,
            row.updated_at,
            row.deleted_at,
        ))
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AuthDomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, display_name, allow_invites, created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_user).transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, AuthDomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, display_name, allow_invites, created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(email.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        row.map(Self::row_to_user).transpose()
    }

    async fn save(&self, user: &User) -> Result<(), AuthDomainError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, display_name, allow_invites, created_at, updated_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                display_name = EXCLUDED.display_name,
                allow_invites = EXCLUDED.allow_invites,
                updated_at = EXCLUDED.updated_at,
                deleted_at = EXCLUDED.deleted_at
            "#,
        )
        .bind(user.id().as_str())
        .bind(user.email().as_str())
        .bind(user.password_hash().map(|h| h.as_str()))
        .bind(user.display_name().map(|d| d.as_str()))
        .bind(user.allow_invites())
        .bind(user.created_at())
        .bind(user.updated_at())
        .bind(user.deleted_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, AuthDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users WHERE email = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(email.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(count.0 > 0)
    }
}
