use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

use super::models::RefreshTokenRow;
use crate::modules::auth::domain::{
    AuthDomainError, RefreshToken, RefreshTokenRepository, TokenId, UserId,
};

/// PostgreSQL implementation of RefreshTokenRepository
pub struct PostgresRefreshTokenRepository {
    pool: Arc<PgPool>,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn row_to_token(row: RefreshTokenRow) -> RefreshToken {
        RefreshToken::reconstruct(
            TokenId::new(row.id),
            UserId::new(row.user_id),
            row.token_hash,
            row.expires_at,
            row.created_at,
            row.revoked_at,
        )
    }
}

#[async_trait]
impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn save(&self, token: &RefreshToken) -> Result<(), AuthDomainError> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(token.id().as_str())
        .bind(token.user_id().as_str())
        .bind(token.token_hash())
        .bind(token.expires_at())
        .bind(token.created_at())
        .bind(token.revoked_at())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: &TokenId) -> Result<Option<RefreshToken>, AuthDomainError> {
        let row: Option<RefreshTokenRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, revoked_at
            FROM refresh_tokens
            WHERE id = $1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(row.map(Self::row_to_token))
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthDomainError> {
        let row: Option<RefreshTokenRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at, revoked_at
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(hash)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(row.map(Self::row_to_token))
    }

    async fn revoke(&self, id: &TokenId) -> Result<(), AuthDomainError> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = $1
            WHERE id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(Utc::now())
        .bind(id.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<(), AuthDomainError> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = $1
            WHERE user_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(Utc::now())
        .bind(user_id.as_str())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, AuthDomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < $1
            "#,
        )
        .bind(Utc::now())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
