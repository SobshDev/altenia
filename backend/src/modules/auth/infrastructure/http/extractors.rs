use axum::http::StatusCode;
use std::sync::Arc;

use crate::modules::auth::application::TokenService;

/// Authenticated user claims extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthClaims {
    pub user_id: String,
    pub email: String,
    pub org_id: Option<String>,
    pub org_role: Option<String>,
}

/// Application state containing token service
pub struct AuthState<TS: TokenService> {
    pub token_service: Arc<TS>,
}

impl<TS: TokenService> Clone for AuthState<TS> {
    fn clone(&self) -> Self {
        Self {
            token_service: self.token_service.clone(),
        }
    }
}

/// Error response for authentication failures
#[derive(Debug)]
pub struct AuthError {
    pub status: StatusCode,
    pub message: String,
}

impl AuthError {
    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }
}

impl From<AuthError> for (StatusCode, String) {
    fn from(err: AuthError) -> Self {
        (err.status, err.message)
    }
}
