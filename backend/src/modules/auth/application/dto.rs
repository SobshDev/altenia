use serde::{Deserialize, Serialize};

// ============================================================================
// Commands (inputs)
// ============================================================================

/// Command to register a new user
#[derive(Debug, Clone)]
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
    pub device_fingerprint: String, // Hash of User-Agent + IP subnet
}

/// Command to login
#[derive(Debug, Clone)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
    pub device_fingerprint: String, // Hash of User-Agent + IP subnet
}

/// Command to logout
#[derive(Debug, Clone)]
pub struct LogoutCommand {
    pub user_id: String,
    pub refresh_token: Option<String>, // If provided, revoke specific token only
}

/// Command to refresh tokens
#[derive(Debug, Clone)]
pub struct RefreshTokenCommand {
    pub refresh_token: String,
    pub device_fingerprint: String, // Hash of User-Agent + IP subnet
}

/// Command to change user's email
#[derive(Debug, Clone)]
pub struct ChangeEmailCommand {
    pub user_id: String,
    pub current_password: String,
    pub new_email: String,
}

/// Command to change user's password
#[derive(Debug, Clone)]
pub struct ChangePasswordCommand {
    pub user_id: String,
    pub current_password: String,
    pub new_password: String,
}

/// Command to delete user's account
#[derive(Debug, Clone)]
pub struct DeleteAccountCommand {
    pub user_id: String,
    pub current_password: String,
}

/// Command to update user's display name
#[derive(Debug, Clone)]
pub struct UpdateDisplayNameCommand {
    pub user_id: String,
    pub display_name: String,
}

// ============================================================================
// Responses (outputs)
// ============================================================================

/// Response after successful authentication
#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64, // seconds until access token expires
}

impl AuthResponse {
    pub fn new(
        user_id: String,
        email: String,
        display_name: Option<String>,
        access_token: String,
        refresh_token: String,
        expires_in: i64,
    ) -> Self {
        Self {
            user_id,
            email,
            display_name,
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
    pub display_name: Option<String>,
    pub created_at: String,
}
