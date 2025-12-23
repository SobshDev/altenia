use async_trait::async_trait;

use crate::modules::auth::domain::{AuthDomainError, UserId};

/// Claims stored in JWT tokens
#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: String,
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

/// Token pair returned after successful authentication
#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_in: i64,  // seconds
    pub refresh_expires_in: i64, // seconds
}

/// Port for JWT token operations
/// Infrastructure layer implements this with jsonwebtoken
#[async_trait]
pub trait TokenService: Send + Sync {
    /// Generate access and refresh token pair
    async fn generate_token_pair(
        &self,
        user_id: &UserId,
        email: &str,
    ) -> Result<TokenPair, AuthDomainError>;

    /// Validate access token and extract claims
    fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthDomainError>;

    /// Decode refresh token and extract claims (also validates)
    fn decode_refresh_token(&self, token: &str) -> Result<TokenClaims, AuthDomainError>;

    /// Hash refresh token for storage (SHA256)
    fn hash_refresh_token(&self, token: &str) -> String;
}

/// Port for ID generation
/// Infrastructure layer implements this with UUID
pub trait IdGenerator: Send + Sync {
    fn generate(&self) -> String;
}
