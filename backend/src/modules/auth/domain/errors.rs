use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthDomainError {
    // Validation errors
    InvalidEmail(String),
    InvalidPassword(String),
    WeakPassword(String),

    // User errors
    UserNotFound,
    UserAlreadyExists,
    InvalidCredentials,

    // Token errors
    TokenExpired,
    TokenInvalid,
    TokenRevoked,

    // Infrastructure errors (will be mapped from infra layer)
    InternalError(String),
}

impl fmt::Display for AuthDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEmail(_) => write!(f, "Invalid email format"),
            Self::InvalidPassword(msg) => write!(f, "Invalid password: {}", msg),
            Self::WeakPassword(reason) => write!(f, "Password is too weak: {}", reason),
            Self::UserNotFound => write!(f, "User not found"),
            Self::UserAlreadyExists => write!(f, "User already exists"),
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::TokenExpired => write!(f, "Token has expired"),
            Self::TokenInvalid => write!(f, "Token is invalid"),
            Self::TokenRevoked => write!(f, "Token has been revoked"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AuthDomainError {}
