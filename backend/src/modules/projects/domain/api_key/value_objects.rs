use crate::modules::projects::domain::errors::ProjectDomainError;

/// API Key ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiKeyId(String);

impl ApiKeyId {
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

impl From<String> for ApiKeyId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// API Key Name - human-readable name for the key (1-100 chars)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyName(String);

impl ApiKeyName {
    const MAX_LENGTH: usize = 100;

    pub fn new(name: String) -> Result<Self, ProjectDomainError> {
        let name = name.trim().to_string();

        if name.is_empty() {
            return Err(ProjectDomainError::InvalidApiKeyName(
                "name cannot be empty".to_string(),
            ));
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(ProjectDomainError::InvalidApiKeyName(format!(
                "name cannot exceed {} characters",
                Self::MAX_LENGTH
            )));
        }

        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// API Key Prefix - first 8 characters of the key for identification
/// Format: "alt_pk_" + first 8 chars of base64 key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyPrefix(String);

impl ApiKeyPrefix {
    const PREFIX: &'static str = "alt_pk_";

    /// Create from the full key (extracts first 8 chars after prefix)
    pub fn from_key(full_key: &str) -> Self {
        // Full key format: alt_pk_{base64_encoded_key}
        // Prefix format: alt_pk_{first_8_chars}
        let key_part = full_key.strip_prefix(Self::PREFIX).unwrap_or(full_key);
        let prefix_chars: String = key_part.chars().take(8).collect();
        Self(format!("{}{}", Self::PREFIX, prefix_chars))
    }

    /// Reconstruct from stored prefix
    pub fn from_string(prefix: String) -> Self {
        Self(prefix)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_api_key_name() {
        assert!(ApiKeyName::new("Production Key".to_string()).is_ok());
        assert!(ApiKeyName::new("  Trimmed  ".to_string()).is_ok());
        assert!(ApiKeyName::new("A".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_api_key_name() {
        assert!(ApiKeyName::new("".to_string()).is_err());
        assert!(ApiKeyName::new("   ".to_string()).is_err());
        assert!(ApiKeyName::new("A".repeat(101)).is_err());
    }

    #[test]
    fn test_api_key_prefix_from_key() {
        let full_key = "alt_pk_abcdefghijklmnop1234567890";
        let prefix = ApiKeyPrefix::from_key(full_key);
        assert_eq!(prefix.as_str(), "alt_pk_abcdefgh");
    }

    #[test]
    fn test_api_key_prefix_short_key() {
        let full_key = "alt_pk_abc";
        let prefix = ApiKeyPrefix::from_key(full_key);
        assert_eq!(prefix.as_str(), "alt_pk_abc");
    }
}
