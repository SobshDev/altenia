use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher as Argon2Hasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash as Argon2Hash,
};
use async_trait::async_trait;

use crate::modules::auth::domain::{AuthDomainError, PasswordHash, PasswordHasher, PlainPassword};

/// Argon2 implementation of PasswordHasher
pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &PlainPassword) -> Result<PasswordHash, AuthDomainError> {
        let salt = SaltString::generate(&mut OsRng);

        let hash = self
            .argon2
            .hash_password(password.as_str().as_bytes(), &salt)
            .map_err(|e| AuthDomainError::InternalError(format!("Failed to hash password: {}", e)))?
            .to_string();

        Ok(PasswordHash::from_hash(hash))
    }

    async fn verify(
        &self,
        password: &PlainPassword,
        hash: &PasswordHash,
    ) -> Result<bool, AuthDomainError> {
        let parsed_hash = Argon2Hash::new(hash.as_str())
            .map_err(|e| AuthDomainError::InternalError(format!("Invalid hash format: {}", e)))?;

        Ok(self
            .argon2
            .verify_password(password.as_str().as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_and_verify() {
        let hasher = Argon2PasswordHasher::new();
        let password = PlainPassword::new("Password123!".to_string()).unwrap();

        let hash = hasher.hash(&password).await.unwrap();

        assert!(hasher.verify(&password, &hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let hasher = Argon2PasswordHasher::new();
        let password = PlainPassword::new("Password123!".to_string()).unwrap();
        let wrong_password = PlainPassword::new("WrongPass456!".to_string()).unwrap();

        let hash = hasher.hash(&password).await.unwrap();

        assert!(!hasher.verify(&wrong_password, &hash).await.unwrap());
    }
}
