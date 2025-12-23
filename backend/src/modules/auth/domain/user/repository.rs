use async_trait::async_trait;

use super::entity::User;
use super::value_objects::{Email, UserId};
use crate::modules::auth::domain::errors::AuthDomainError;

/// Port for user persistence operations
/// Infrastructure layer implements this with PostgreSQL
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by ID
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AuthDomainError>;

    /// Find user by email
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, AuthDomainError>;

    /// Save a user (insert or update)
    async fn save(&self, user: &User) -> Result<(), AuthDomainError>;

    /// Check if email is already registered
    async fn exists_by_email(&self, email: &Email) -> Result<bool, AuthDomainError>;
}
