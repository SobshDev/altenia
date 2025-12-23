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

    /// Update the user's email
    pub fn update_email(&mut self, new_email: Email) {
        self.email = new_email;
        self.updated_at = Utc::now();
    }

    /// Update the user's password
    pub fn update_password(&mut self, new_password_hash: PasswordHash) {
        self.password_hash = Some(new_password_hash);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::auth::domain::user::value_objects::PasswordHash;

    fn create_test_user_id() -> UserId {
        UserId::new("test-user-id".to_string())
    }

    fn create_test_email() -> Email {
        Email::new("test@example.com".to_string()).unwrap()
    }

    fn create_test_password_hash() -> PasswordHash {
        PasswordHash::from_hash("hashed_password".to_string())
    }

    #[test]
    fn test_new_user_with_password() {
        let id = create_test_user_id();
        let email = create_test_email();
        let password_hash = create_test_password_hash();

        let user = User::new(id.clone(), email.clone(), password_hash);

        assert_eq!(user.id().as_str(), "test-user-id");
        assert_eq!(user.email().as_str(), "test@example.com");
        assert!(user.has_password());
        assert!(user.password_hash().is_some());
    }

    #[test]
    fn test_new_oauth_user_without_password() {
        let id = create_test_user_id();
        let email = create_test_email();

        let user = User::new_oauth(id, email);

        assert_eq!(user.id().as_str(), "test-user-id");
        assert_eq!(user.email().as_str(), "test@example.com");
        assert!(!user.has_password());
        assert!(user.password_hash().is_none());
    }

    #[test]
    fn test_user_timestamps_are_set() {
        let id = create_test_user_id();
        let email = create_test_email();
        let password_hash = create_test_password_hash();

        let before = Utc::now();
        let user = User::new(id, email, password_hash);
        let after = Utc::now();

        assert!(user.created_at() >= before && user.created_at() <= after);
        assert!(user.updated_at() >= before && user.updated_at() <= after);
    }

    #[test]
    fn test_reconstruct_user() {
        let id = create_test_user_id();
        let email = create_test_email();
        let password_hash = Some(create_test_password_hash());
        let created_at = Utc::now();
        let updated_at = Utc::now();

        let user = User::reconstruct(id, email, password_hash, created_at, updated_at);

        assert_eq!(user.id().as_str(), "test-user-id");
        assert!(user.has_password());
        assert_eq!(user.created_at(), created_at);
        assert_eq!(user.updated_at(), updated_at);
    }
}
