use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::modules::auth::application::ports::{OrgContext, TokenClaims, TokenPair, TokenService};
use crate::modules::auth::domain::{AuthDomainError, UserId};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,                 // user_id
    email: String,               // user email
    org_id: Option<String>,      // organization id
    org_role: Option<String>,    // role in organization
    exp: i64,                    // expiration time
    iat: i64,                    // issued at
    token_type: String,          // "access" or "refresh"
}

/// JWT token service configuration
pub struct JwtConfig {
    pub access_secret: String,
    pub refresh_secret: String,
    pub access_expiry_secs: i64,
    pub refresh_expiry_secs: i64,
}

impl JwtConfig {
    pub fn new(
        access_secret: String,
        refresh_secret: String,
        access_expiry_secs: i64,
        refresh_expiry_secs: i64,
    ) -> Self {
        Self {
            access_secret,
            refresh_secret,
            access_expiry_secs,
            refresh_expiry_secs,
        }
    }

    /// Default configuration with 15 min access and 7 day refresh
    pub fn default_with_secrets(access_secret: String, refresh_secret: String) -> Self {
        Self {
            access_secret,
            refresh_secret,
            access_expiry_secs: 15 * 60,        // 15 minutes
            refresh_expiry_secs: 7 * 24 * 60 * 60, // 7 days
        }
    }
}

/// JWT implementation of TokenService
pub struct JwtTokenService {
    config: JwtConfig,
}

impl JwtTokenService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl TokenService for JwtTokenService {
    async fn generate_token_pair(
        &self,
        user_id: &UserId,
        email: &str,
        org_context: Option<OrgContext>,
    ) -> Result<TokenPair, AuthDomainError> {
        let now = Utc::now().timestamp();

        let (org_id, org_role) = match org_context {
            Some(ctx) => (Some(ctx.org_id), Some(ctx.org_role)),
            None => (None, None),
        };

        // Generate access token
        let access_claims = Claims {
            sub: user_id.as_str().to_string(),
            email: email.to_string(),
            org_id: org_id.clone(),
            org_role: org_role.clone(),
            exp: now + self.config.access_expiry_secs,
            iat: now,
            token_type: "access".to_string(),
        };

        let access_token = encode(
            &Header::new(Algorithm::HS256),
            &access_claims,
            &EncodingKey::from_secret(self.config.access_secret.as_bytes()),
        )
        .map_err(|e| AuthDomainError::InternalError(format!("Failed to create access token: {}", e)))?;

        // Generate refresh token
        let refresh_claims = Claims {
            sub: user_id.as_str().to_string(),
            email: email.to_string(),
            org_id,
            org_role,
            exp: now + self.config.refresh_expiry_secs,
            iat: now,
            token_type: "refresh".to_string(),
        };

        let refresh_token = encode(
            &Header::new(Algorithm::HS256),
            &refresh_claims,
            &EncodingKey::from_secret(self.config.refresh_secret.as_bytes()),
        )
        .map_err(|e| AuthDomainError::InternalError(format!("Failed to create refresh token: {}", e)))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            access_expires_in: self.config.access_expiry_secs,
            refresh_expires_in: self.config.refresh_expiry_secs,
        })
    }

    fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthDomainError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.access_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthDomainError::TokenExpired,
            _ => AuthDomainError::TokenInvalid,
        })?;

        if token_data.claims.token_type != "access" {
            return Err(AuthDomainError::TokenInvalid);
        }

        Ok(TokenClaims {
            user_id: token_data.claims.sub,
            email: token_data.claims.email,
            org_id: token_data.claims.org_id,
            org_role: token_data.claims.org_role,
            exp: token_data.claims.exp,
            iat: token_data.claims.iat,
        })
    }

    fn decode_refresh_token(&self, token: &str) -> Result<TokenClaims, AuthDomainError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.refresh_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthDomainError::TokenExpired,
            _ => AuthDomainError::TokenInvalid,
        })?;

        if token_data.claims.token_type != "refresh" {
            return Err(AuthDomainError::TokenInvalid);
        }

        Ok(TokenClaims {
            user_id: token_data.claims.sub,
            email: token_data.claims.email,
            org_id: token_data.claims.org_id,
            org_role: token_data.claims.org_role,
            exp: token_data.claims.exp,
            iat: token_data.claims.iat,
        })
    }

    fn hash_refresh_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> JwtConfig {
        JwtConfig::new(
            "test_access_secret".to_string(),
            "test_refresh_secret".to_string(),
            900,  // 15 minutes
            604800, // 7 days
        )
    }

    fn create_test_service() -> JwtTokenService {
        JwtTokenService::new(create_test_config())
    }

    #[tokio::test]
    async fn test_generate_token_pair() {
        let service = create_test_service();
        let user_id = UserId::new("user-123".to_string());

        let result = service.generate_token_pair(&user_id, "test@example.com", None).await;

        assert!(result.is_ok());
        let token_pair = result.unwrap();
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.access_expires_in, 900);
        assert_eq!(token_pair.refresh_expires_in, 604800);
    }

    #[tokio::test]
    async fn test_generate_token_pair_with_org_context() {
        let service = create_test_service();
        let user_id = UserId::new("user-123".to_string());
        let org_context = Some(OrgContext {
            org_id: "org-456".to_string(),
            org_role: "owner".to_string(),
        });

        let result = service.generate_token_pair(&user_id, "test@example.com", org_context).await;

        assert!(result.is_ok());
        let token_pair = result.unwrap();

        // Validate the token contains org claims
        let claims = service.validate_access_token(&token_pair.access_token).unwrap();
        assert_eq!(claims.org_id, Some("org-456".to_string()));
        assert_eq!(claims.org_role, Some("owner".to_string()));
    }

    #[tokio::test]
    async fn test_validate_access_token_success() {
        let service = create_test_service();
        let user_id = UserId::new("user-123".to_string());
        let email = "test@example.com";

        let token_pair = service.generate_token_pair(&user_id, email, None).await.unwrap();
        let claims = service.validate_access_token(&token_pair.access_token);

        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.user_id, "user-123");
        assert_eq!(claims.email, email);
        assert!(claims.org_id.is_none());
        assert!(claims.org_role.is_none());
    }

    #[tokio::test]
    async fn test_validate_access_token_with_refresh_token_fails() {
        let service = create_test_service();
        let user_id = UserId::new("user-123".to_string());

        let token_pair = service.generate_token_pair(&user_id, "test@example.com", None).await.unwrap();
        let result = service.validate_access_token(&token_pair.refresh_token);

        assert!(matches!(result, Err(AuthDomainError::TokenInvalid)));
    }

    #[tokio::test]
    async fn test_validate_access_token_invalid_signature() {
        let service = create_test_service();
        let result = service.validate_access_token("invalid.token.here");

        assert!(matches!(result, Err(AuthDomainError::TokenInvalid)));
    }

    #[tokio::test]
    async fn test_decode_refresh_token_success() {
        let service = create_test_service();
        let user_id = UserId::new("user-456".to_string());
        let email = "refresh@example.com";

        let token_pair = service.generate_token_pair(&user_id, email, None).await.unwrap();
        let claims = service.decode_refresh_token(&token_pair.refresh_token);

        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.user_id, "user-456");
        assert_eq!(claims.email, email);
    }

    #[tokio::test]
    async fn test_decode_refresh_token_with_access_token_fails() {
        let service = create_test_service();
        let user_id = UserId::new("user-123".to_string());

        let token_pair = service.generate_token_pair(&user_id, "test@example.com", None).await.unwrap();
        let result = service.decode_refresh_token(&token_pair.access_token);

        assert!(matches!(result, Err(AuthDomainError::TokenInvalid)));
    }

    #[test]
    fn test_hash_refresh_token_consistency() {
        let service = create_test_service();
        let token = "my_refresh_token";

        let hash1 = service.hash_refresh_token(token);
        let hash2 = service.hash_refresh_token(token);

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_hash_refresh_token_different_inputs() {
        let service = create_test_service();

        let hash1 = service.hash_refresh_token("token_a");
        let hash2 = service.hash_refresh_token("token_b");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_jwt_config_default_with_secrets() {
        let config = JwtConfig::default_with_secrets(
            "access".to_string(),
            "refresh".to_string(),
        );

        assert_eq!(config.access_secret, "access");
        assert_eq!(config.refresh_secret, "refresh");
        assert_eq!(config.access_expiry_secs, 15 * 60);
        assert_eq!(config.refresh_expiry_secs, 7 * 24 * 60 * 60);
    }
}
