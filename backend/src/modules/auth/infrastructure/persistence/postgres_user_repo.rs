use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::UserRow;
use crate::modules::auth::domain::{
    AuthDomainError, Email, PasswordHash, User, UserId, UserRepository,
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

        Ok(User::reconstruct(
            user_id,
            email,
            password_hash,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AuthDomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE id = $1
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
            SELECT id, email, password_hash, created_at, updated_at
            FROM users
            WHERE email = $1
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
            INSERT INTO users (id, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(user.id().as_str())
        .bind(user.email().as_str())
        .bind(user.password_hash().map(|h| h.as_str()))
        .bind(user.created_at())
        .bind(user.updated_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, AuthDomainError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users WHERE email = $1
            "#,
        )
        .bind(email.as_str())
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(count.0 > 0)
    }
}
