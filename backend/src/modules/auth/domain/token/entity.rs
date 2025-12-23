use chrono::{DateTime, Utc};

use crate::modules::auth::domain::user::UserId;

/// Token ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenId(String);

impl TokenId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for TokenId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Refresh token entity
/// Stored in database to enable token revocation and rotation
#[derive(Debug, Clone)]
pub struct RefreshToken {
    id: TokenId,
    user_id: UserId,
    token_hash: String, // SHA256 hash of the actual token
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
    /// Create a new refresh token
    pub fn new(
        id: TokenId,
        user_id: UserId,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            token_hash,
            expires_at,
            created_at: Utc::now(),
            revoked_at: None,
        }
    }

    /// Reconstruct from persistence layer
    pub fn reconstruct(
        id: TokenId,
        user_id: UserId,
        token_hash: String,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        revoked_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            user_id,
            token_hash,
            expires_at,
            created_at,
            revoked_at,
        }
    }

    // Getters
    pub fn id(&self) -> &TokenId {
        &self.id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn token_hash(&self) -> &str {
        &self.token_hash
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }

    // Domain behavior
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked()
    }

    /// Revoke this token
    pub fn revoke(&mut self) {
        if self.revoked_at.is_none() {
            self.revoked_at = Some(Utc::now());
        }
    }
}
