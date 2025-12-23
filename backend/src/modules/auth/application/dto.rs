use serde::{Deserialize, Serialize};

// ============================================================================
// Commands (inputs)
// ============================================================================

/// Command to register a new user
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
}

/// Command to login
#[derive(Debug, Clone, Deserialize)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

/// Command to logout
#[derive(Debug, Clone)]
pub struct LogoutCommand {
    pub user_id: String,
    pub refresh_token: Option<String>, // If provided, revoke specific token only
}

/// Command to refresh tokens
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenCommand {
    pub refresh_token: String,
}

// ============================================================================
// Responses (outputs)
// ============================================================================

/// Response after successful authentication
#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64, // seconds until access token expires
}

impl AuthResponse {
    pub fn new(
        user_id: String,
        email: String,
        access_token: String,
        refresh_token: String,
        expires_in: i64,
    ) -> Self {
        Self {
            user_id,
            email,
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

/// User data transfer object
#[derive(Debug, Clone, Serialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub created_at: String,
}
