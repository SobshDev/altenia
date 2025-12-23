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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_token_id() -> TokenId {
        TokenId::new("test-token-id".to_string())
    }

    fn create_test_user_id() -> UserId {
        UserId::new("test-user-id".to_string())
    }

    #[test]
    fn test_new_refresh_token() {
        let token_id = create_test_token_id();
        let user_id = create_test_user_id();
        let token_hash = "hashed_token".to_string();
        let expires_at = Utc::now() + Duration::days(7);

        let token = RefreshToken::new(token_id, user_id, token_hash.clone(), expires_at);

        assert_eq!(token.id().as_str(), "test-token-id");
        assert_eq!(token.user_id().as_str(), "test-user-id");
        assert_eq!(token.token_hash(), token_hash);
        assert_eq!(token.expires_at(), expires_at);
        assert!(token.revoked_at().is_none());
    }

    #[test]
    fn test_token_is_valid_when_not_expired_and_not_revoked() {
        let token = RefreshToken::new(
            create_test_token_id(),
            create_test_user_id(),
            "hash".to_string(),
            Utc::now() + Duration::days(7),
        );

        assert!(token.is_valid());
        assert!(!token.is_expired());
        assert!(!token.is_revoked());
    }

    #[test]
    fn test_token_is_expired() {
        let token = RefreshToken::new(
            create_test_token_id(),
            create_test_user_id(),
            "hash".to_string(),
            Utc::now() - Duration::seconds(1), // expired 1 second ago
        );

        assert!(token.is_expired());
        assert!(!token.is_valid());
    }

    #[test]
    fn test_token_revocation() {
        let mut token = RefreshToken::new(
            create_test_token_id(),
            create_test_user_id(),
            "hash".to_string(),
            Utc::now() + Duration::days(7),
        );

        assert!(!token.is_revoked());
        assert!(token.is_valid());

        token.revoke();

        assert!(token.is_revoked());
        assert!(!token.is_valid());
        assert!(token.revoked_at().is_some());
    }

    #[test]
    fn test_double_revocation_keeps_original_timestamp() {
        let mut token = RefreshToken::new(
            create_test_token_id(),
            create_test_user_id(),
            "hash".to_string(),
            Utc::now() + Duration::days(7),
        );

        token.revoke();
        let first_revoked_at = token.revoked_at().unwrap();

        // Small delay to ensure timestamps would differ
        std::thread::sleep(std::time::Duration::from_millis(10));

        token.revoke();
        let second_revoked_at = token.revoked_at().unwrap();

        assert_eq!(first_revoked_at, second_revoked_at);
    }

    #[test]
    fn test_reconstruct_token() {
        let token_id = create_test_token_id();
        let user_id = create_test_user_id();
        let token_hash = "hash".to_string();
        let expires_at = Utc::now() + Duration::days(7);
        let created_at = Utc::now() - Duration::hours(1);
        let revoked_at = Some(Utc::now());

        let token = RefreshToken::reconstruct(
            token_id,
            user_id,
            token_hash.clone(),
            expires_at,
            created_at,
            revoked_at,
        );

        assert_eq!(token.id().as_str(), "test-token-id");
        assert_eq!(token.token_hash(), token_hash);
        assert_eq!(token.created_at(), created_at);
        assert!(token.is_revoked());
    }

    #[test]
    fn test_token_id_from_string() {
        let id: TokenId = "my-token-id".to_string().into();
        assert_eq!(id.as_str(), "my-token-id");
        assert_eq!(id.into_inner(), "my-token-id");
    }
}
