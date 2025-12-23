pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use application::{AuthResponse, AuthService, LoginCommand, LogoutCommand, RefreshTokenCommand, RegisterUserCommand};
pub use domain::{AuthDomainError, Email, PasswordHash, PlainPassword, User, UserId};
pub use infrastructure::{
    auth_routes, Argon2PasswordHasher, AuthClaims, JwtConfig, JwtTokenService,
    PostgresRefreshTokenRepository, PostgresUserRepository, UuidGenerator,
};
