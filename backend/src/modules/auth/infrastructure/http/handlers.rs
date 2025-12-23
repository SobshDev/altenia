use axum::{
    extract::State,
    http::{header::USER_AGENT, HeaderMap, StatusCode},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use super::extractors::AuthClaims;
use crate::modules::auth::application::{
    AuthResponse, AuthService, ChangeEmailCommand, ChangePasswordCommand, LoginCommand,
    LogoutCommand, RefreshTokenCommand, RegisterUserCommand,
};
use crate::modules::auth::domain::{
    AuthDomainError, PasswordHasher, RefreshTokenRepository, UserRepository,
};
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};

/// Generate device fingerprint from User-Agent and X-Forwarded-For headers
/// Uses /24 subnet for IPv4 to allow for NAT variations
fn generate_device_fingerprint(headers: &HeaderMap) -> String {
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Try to get IP from X-Forwarded-For header (common in proxied setups)
    let ip_str = headers
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .unwrap_or("unknown");

    // Extract /24 subnet for IPv4 or use as-is for others
    let ip_subnet = if let Some(ip) = ip_str.parse::<std::net::IpAddr>().ok() {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            }
            std::net::IpAddr::V6(ipv6) => {
                let segments = ipv6.segments();
                format!("{:x}:{:x}:{:x}::/48", segments[0], segments[1], segments[2])
            }
        }
    } else {
        ip_str.to_string()
    };

    let mut hasher = Sha256::new();
    hasher.update(user_agent.as_bytes());
    hasher.update(ip_subnet.as_bytes());
    format!("{:x}", hasher.finalize())
}

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

#[derive(Debug, Deserialize)]
pub struct ChangeEmailRequest {
    pub current_password: String,
    pub new_email: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
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
        AuthDomainError::WeakPassword(ref reason) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Password is too weak: {}", reason),
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
        AuthDomainError::NoPasswordSet => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "This account does not have a password set".to_string(),
                code: "NO_PASSWORD_SET".to_string(),
            }),
        ),
        AuthDomainError::EmailAlreadyInUse => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Email is already in use".to_string(),
                code: "EMAIL_IN_USE".to_string(),
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
        AuthDomainError::InternalError(ref msg) => {
            tracing::error!(error = %msg, "Internal error occurred");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "An internal error occurred. Please try again later.".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
        }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/auth/register
pub async fn register<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    let device_fingerprint = generate_device_fingerprint(&headers);

    let cmd = RegisterUserCommand {
        email: req.email,
        password: req.password,
        device_fingerprint,
    };

    auth_service
        .register(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/auth/login
pub async fn login<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    let device_fingerprint = generate_device_fingerprint(&headers);

    let cmd = LoginCommand {
        email: req.email,
        password: req.password,
        device_fingerprint,
    };

    auth_service
        .login(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/auth/refresh
pub async fn refresh<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    headers: HeaderMap,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    let device_fingerprint = generate_device_fingerprint(&headers);

    let cmd = RefreshTokenCommand {
        refresh_token: req.refresh_token,
        device_fingerprint,
    };

    auth_service
        .refresh(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/auth/logout (protected)
pub async fn logout<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<LogoutRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
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
pub async fn me<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<UserResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
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

/// PATCH /api/auth/me/email (protected)
pub async fn change_email<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<ChangeEmailRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    let cmd = ChangeEmailCommand {
        user_id: claims.user_id,
        current_password: req.current_password,
        new_email: req.new_email,
    };

    auth_service
        .change_email(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// PATCH /api/auth/me/password (protected)
pub async fn change_password<U, T, P, TS, ID, OR, MR>(
    State(auth_service): State<Arc<AuthService<U, T, P, TS, ID, OR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    let cmd = ChangePasswordCommand {
        user_id: claims.user_id,
        current_password: req.current_password,
        new_password: req.new_password,
    };

    auth_service
        .change_password(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}
