use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::modules::auth::application::ports::{TokenClaims, TokenPair, TokenService};
use crate::modules::auth::domain::{AuthDomainError, UserId};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,        // user_id
    email: String,      // user email
    exp: i64,           // expiration time
    iat: i64,           // issued at
    token_type: String, // "access" or "refresh"
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
    ) -> Result<TokenPair, AuthDomainError> {
        let now = Utc::now().timestamp();

        // Generate access token
        let access_claims = Claims {
            sub: user_id.as_str().to_string(),
            email: email.to_string(),
            exp: now + self.config.access_expiry_secs,
            iat: now,
            token_type: "access".to_string(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.config.access_secret.as_bytes()),
        )
        .map_err(|e| AuthDomainError::InternalError(format!("Failed to create access token: {}", e)))?;

        // Generate refresh token
        let refresh_claims = Claims {
            sub: user_id.as_str().to_string(),
            email: email.to_string(),
            exp: now + self.config.refresh_expiry_secs,
            iat: now,
            token_type: "refresh".to_string(),
        };

        let refresh_token = encode(
            &Header::default(),
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
