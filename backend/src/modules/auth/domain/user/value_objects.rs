use crate::modules::auth::domain::errors::AuthDomainError;

/// User ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
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

impl From<String> for UserId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Email - validated email value object
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, AuthDomainError> {
        let email = email.trim().to_lowercase();

        // Basic validation: must contain @ and have content on both sides
        if let Some(at_pos) = email.find('@') {
            let (local, domain) = email.split_at(at_pos);
            let domain = &domain[1..]; // Skip the @

            if !local.is_empty() && !domain.is_empty() && domain.contains('.') {
                return Ok(Self(email));
            }
        }

        Err(AuthDomainError::InvalidEmail(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Password hash - opaque wrapper for hashed password
/// Domain doesn't know the hashing algorithm (argon2, bcrypt, etc.)
#[derive(Debug, Clone)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Plain password - for validation before hashing
/// Never stored, only used in transit
#[derive(Debug, Clone)]
pub struct PlainPassword(String);

impl PlainPassword {
    pub fn new(password: String) -> Result<Self, AuthDomainError> {
        // Password strength validation
        if password.len() < 8 {
            return Err(AuthDomainError::WeakPassword);
        }

        Ok(Self(password))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return inner string (for hashing)
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(Email::new("test@example.com".to_string()).is_ok());
        assert!(Email::new("USER@EXAMPLE.COM".to_string()).is_ok());
        assert!(Email::new("  test@example.com  ".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(Email::new("invalid".to_string()).is_err());
        assert!(Email::new("@example.com".to_string()).is_err());
        assert!(Email::new("test@".to_string()).is_err());
        assert!(Email::new("test@example".to_string()).is_err());
    }

    #[test]
    fn test_valid_password() {
        assert!(PlainPassword::new("password123".to_string()).is_ok());
        assert!(PlainPassword::new("12345678".to_string()).is_ok());
    }

    #[test]
    fn test_weak_password() {
        assert!(PlainPassword::new("short".to_string()).is_err());
        assert!(PlainPassword::new("1234567".to_string()).is_err());
    }
}
