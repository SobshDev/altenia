use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::extractors::AuthClaims;
use crate::modules::auth::application::{
    AuthResponse, AuthService, LoginCommand, LogoutCommand, RefreshTokenCommand,
    RegisterUserCommand, UserDto,
};
use crate::modules::auth::domain::{
    AuthDomainError, PasswordHasher, RefreshTokenRepository, UserRepository,
};
use crate::modules::auth::application::ports::{IdGenerator, TokenService};

// ============================================================================
// Request/Response DTOs for HTTP layer
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub user_id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct UserResponseDto {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl From<AuthResponse> for AuthResponseDto {
    fn from(r: AuthResponse) -> Self {
        Self {
            user_id: r.user_id,
            email: r.email,
            access_token: r.access_token,
            refresh_token: r.refresh_token,
            token_type: r.token_type,
            expires_in: r.expires_in,
        }
    }
}

// ============================================================================
// Error handling
// ============================================================================

fn to_error_response(e: AuthDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        AuthDomainError::InvalidEmail(_) | AuthDomainError::InvalidPassword(_) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        AuthDomainError::WeakPassword => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "WEAK_PASSWORD".to_string(),
            }),
        ),
        AuthDomainError::UserAlreadyExists => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "User already exists".to_string(),
                code: "USER_EXISTS".to_string(),
            }),
        ),
        AuthDomainError::UserNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
            }),
        ),
        AuthDomainError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
                code: "INVALID_CREDENTIALS".to_string(),
            }),
        ),
        AuthDomainError::TokenExpired => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Token has expired".to_string(),
                code: "TOKEN_EXPIRED".to_string(),
            }),
        ),
        AuthDomainError::TokenInvalid | AuthDomainError::TokenRevoked => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid token".to_string(),
                code: "INVALID_TOKEN".to_string(),
            }),
        ),
        AuthDomainError::InternalError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: msg,
                code: "INTERNAL_ERROR".to_string(),
            }),
        ),
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/auth/register
pub async fn register<U, T, P, TS, ID>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID>>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    let cmd = RegisterUserCommand {
        email: req.email,
        password: req.password,
    };

    auth_service
        .register(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/auth/login
pub async fn login<U, T, P, TS, ID>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID>>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    let cmd = LoginCommand {
        email: req.email,
        password: req.password,
    };

    auth_service
        .login(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/auth/refresh
pub async fn refresh<U, T, P, TS, ID>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID>>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    let cmd = RefreshTokenCommand {
        refresh_token: req.refresh_token,
    };

    auth_service
        .refresh(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/auth/logout (protected)
pub async fn logout<U, T, P, TS, ID>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<LogoutRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    let cmd = LogoutCommand {
        user_id: claims.user_id,
        refresh_token: req.refresh_token,
    };

    auth_service
        .logout(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// GET /api/auth/me (protected)
pub async fn me<U, T, P, TS, ID>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID>>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<UserResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    auth_service
        .get_current_user(&claims.user_id)
        .await
        .map(|user| {
            Json(UserResponseDto {
                id: user.id().as_str().to_string(),
                email: user.email().as_str().to_string(),
            })
        })
        .map_err(to_error_response)
}
