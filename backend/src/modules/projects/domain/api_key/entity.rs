use chrono::{DateTime, Utc};

use super::value_objects::{ApiKeyId, ApiKeyName, ApiKeyPrefix};
use crate::modules::projects::domain::project::ProjectId;

/// ApiKey - entity for authenticating ingestion requests
#[derive(Debug, Clone)]
pub struct ApiKey {
    id: ApiKeyId,
    project_id: ProjectId,
    name: ApiKeyName,
    key_prefix: ApiKeyPrefix,
    key_hash: String,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

impl ApiKey {
    /// Create a new API key
    pub fn new(
        id: ApiKeyId,
        project_id: ProjectId,
        name: ApiKeyName,
        key_prefix: ApiKeyPrefix,
        key_hash: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            key_prefix,
            key_hash,
            created_at: Utc::now(),
            expires_at,
            revoked_at: None,
        }
    }

    /// Reconstruct from persistence layer
    pub fn reconstruct(
        id: ApiKeyId,
        project_id: ProjectId,
        name: ApiKeyName,
        key_prefix: ApiKeyPrefix,
        key_hash: String,
        created_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        revoked_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            key_prefix,
            key_hash,
            created_at,
            expires_at,
            revoked_at,
        }
    }

    // Getters
    pub fn id(&self) -> &ApiKeyId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn name(&self) -> &ApiKeyName {
        &self.name
    }

    pub fn key_prefix(&self) -> &ApiKeyPrefix {
        &self.key_prefix
    }

    pub fn key_hash(&self) -> &str {
        &self.key_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    pub fn revoked_at(&self) -> Option<DateTime<Utc>> {
        self.revoked_at
    }

    // Domain behavior
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked()
    }

    /// Revoke this API key
    pub fn revoke(&mut self) {
        if self.revoked_at.is_none() {
            self.revoked_at = Some(Utc::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_api_key() -> ApiKey {
        let id = ApiKeyId::new("key-123".to_string());
        let project_id = ProjectId::new("proj-456".to_string());
        let name = ApiKeyName::new("Production Key".to_string()).unwrap();
        let prefix = ApiKeyPrefix::from_key("alt_pk_abcdefgh12345678");
        let hash = "sha256_hash_here".to_string();
        ApiKey::new(id, project_id, name, prefix, hash, None)
    }

    #[test]
    fn test_new_api_key() {
        let key = create_test_api_key();
        assert!(!key.is_expired());
        assert!(!key.is_revoked());
        assert!(key.is_valid());
        assert_eq!(key.name().as_str(), "Production Key");
    }

    #[test]
    fn test_api_key_with_expiry() {
        let id = ApiKeyId::new("key-123".to_string());
        let project_id = ProjectId::new("proj-456".to_string());
        let name = ApiKeyName::new("Temp Key".to_string()).unwrap();
        let prefix = ApiKeyPrefix::from_key("alt_pk_abcdefgh12345678");
        let hash = "sha256_hash_here".to_string();
        let expires_at = Utc::now() + chrono::Duration::days(30);

        let key = ApiKey::new(id, project_id, name, prefix, hash, Some(expires_at));
        assert!(!key.is_expired());
        assert!(key.is_valid());
    }

    #[test]
    fn test_expired_api_key() {
        let id = ApiKeyId::new("key-123".to_string());
        let project_id = ProjectId::new("proj-456".to_string());
        let name = ApiKeyName::new("Expired Key".to_string()).unwrap();
        let prefix = ApiKeyPrefix::from_key("alt_pk_abcdefgh12345678");
        let hash = "sha256_hash_here".to_string();
        let expires_at = Utc::now() - chrono::Duration::days(1);

        let key = ApiKey::new(id, project_id, name, prefix, hash, Some(expires_at));
        assert!(key.is_expired());
        assert!(!key.is_valid());
    }

    #[test]
    fn test_revoke_api_key() {
        let mut key = create_test_api_key();
        assert!(!key.is_revoked());

        key.revoke();
        assert!(key.is_revoked());
        assert!(!key.is_valid());
    }

    #[test]
    fn test_revoke_is_idempotent() {
        let mut key = create_test_api_key();
        key.revoke();
        let first_revoked_at = key.revoked_at();

        key.revoke();
        assert_eq!(key.revoked_at(), first_revoked_at);
    }
}
