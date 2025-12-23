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
        // Minimum 8 characters
        if password.len() < 8 {
            return Err(AuthDomainError::WeakPassword(
                "must be at least 8 characters".to_string(),
            ));
        }

        // Must contain uppercase
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthDomainError::WeakPassword(
                "must contain at least one uppercase letter".to_string(),
            ));
        }

        // Must contain lowercase
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthDomainError::WeakPassword(
                "must contain at least one lowercase letter".to_string(),
            ));
        }

        // Must contain digit
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(AuthDomainError::WeakPassword(
                "must contain at least one digit".to_string(),
            ));
        }

        // Must contain special character
        const SPECIAL_CHARS: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
        if !password.chars().any(|c| SPECIAL_CHARS.contains(c)) {
            return Err(AuthDomainError::WeakPassword(
                "must contain at least one special character".to_string(),
            ));
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
        assert!(PlainPassword::new("Password1!".to_string()).is_ok());
        assert!(PlainPassword::new("Str0ng@Pass".to_string()).is_ok());
        assert!(PlainPassword::new("Complex1$Password".to_string()).is_ok());
    }

    #[test]
    fn test_weak_password_too_short() {
        let result = PlainPassword::new("Pass1!".to_string());
        assert!(matches!(result, Err(AuthDomainError::WeakPassword(msg)) if msg.contains("8 characters")));
    }

    #[test]
    fn test_weak_password_no_uppercase() {
        let result = PlainPassword::new("password1!".to_string());
        assert!(matches!(result, Err(AuthDomainError::WeakPassword(msg)) if msg.contains("uppercase")));
    }

    #[test]
    fn test_weak_password_no_lowercase() {
        let result = PlainPassword::new("PASSWORD1!".to_string());
        assert!(matches!(result, Err(AuthDomainError::WeakPassword(msg)) if msg.contains("lowercase")));
    }

    #[test]
    fn test_weak_password_no_digit() {
        let result = PlainPassword::new("Password!".to_string());
        assert!(matches!(result, Err(AuthDomainError::WeakPassword(msg)) if msg.contains("digit")));
    }

    #[test]
    fn test_weak_password_no_special_char() {
        let result = PlainPassword::new("Password1".to_string());
        assert!(matches!(result, Err(AuthDomainError::WeakPassword(msg)) if msg.contains("special")));
    }
}
