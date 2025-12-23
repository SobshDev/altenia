use async_trait::async_trait;

use super::entity::{RefreshToken, TokenId};
use crate::modules::auth::domain::errors::AuthDomainError;
use crate::modules::auth::domain::user::UserId;

/// Port for refresh token persistence operations
/// Infrastructure layer implements this with PostgreSQL
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    /// Save a refresh token
    async fn save(&self, token: &RefreshToken) -> Result<(), AuthDomainError>;

    /// Find token by ID
    async fn find_by_id(&self, id: &TokenId) -> Result<Option<RefreshToken>, AuthDomainError>;

    /// Find token by hash
    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthDomainError>;

    /// Revoke a specific token
    async fn revoke(&self, id: &TokenId) -> Result<(), AuthDomainError>;

    /// Revoke all tokens for a user (logout everywhere)
    async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<(), AuthDomainError>;

    /// Delete expired tokens (cleanup)
    async fn delete_expired(&self) -> Result<u64, AuthDomainError>;
}
