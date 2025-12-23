use chrono::{DateTime, Utc};

use super::value_objects::{Email, PasswordHash, UserId};

/// User aggregate root
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,
    password_hash: Option<PasswordHash>, // None for OAuth-only users
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with email/password authentication
    pub fn new(id: UserId, email: Email, password_hash: PasswordHash) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash: Some(password_hash),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new OAuth-only user (no password)
    pub fn new_oauth(id: UserId, email: Email) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct user from persistence layer
    pub fn reconstruct(
        id: UserId,
        email: Email,
        password_hash: Option<PasswordHash>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> Option<&PasswordHash> {
        self.password_hash.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Domain behavior
    pub fn has_password(&self) -> bool {
        self.password_hash.is_some()
    }
}
