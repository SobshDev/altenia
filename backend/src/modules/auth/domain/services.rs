use async_trait::async_trait;

use super::errors::AuthDomainError;
use super::user::value_objects::{PasswordHash, PlainPassword};

/// Port for password hashing operations
/// Infrastructure layer implements this with argon2
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hash a plain password
    async fn hash(&self, password: &PlainPassword) -> Result<PasswordHash, AuthDomainError>;

    /// Verify a plain password against a hash
    async fn verify(
        &self,
        password: &PlainPassword,
        hash: &PasswordHash,
    ) -> Result<bool, AuthDomainError>;
}
