use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::modules::auth::application::dto::{
    AuthResponse, LoginCommand, LogoutCommand, RefreshTokenCommand, RegisterUserCommand,
};
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::domain::{
    AuthDomainError, Email, PasswordHash, PasswordHasher, PlainPassword, RefreshToken,
    RefreshTokenRepository, TokenId, User, UserId, UserRepository,
};

/// Authentication service - orchestrates all auth use cases
pub struct AuthService<U, T, P, TS, ID>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    user_repo: Arc<U>,
    token_repo: Arc<T>,
    password_hasher: Arc<P>,
    token_service: Arc<TS>,
    id_generator: Arc<ID>,
    refresh_token_duration_days: i64,
}

impl<U, T, P, TS, ID> AuthService<U, T, P, TS, ID>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
{
    pub fn new(
        user_repo: Arc<U>,
        token_repo: Arc<T>,
        password_hasher: Arc<P>,
        token_service: Arc<TS>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            user_repo,
            token_repo,
            password_hasher,
            token_service,
            id_generator,
            refresh_token_duration_days: 7,
        }
    }

    /// Register a new user
    pub async fn register(&self, cmd: RegisterUserCommand) -> Result<AuthResponse, AuthDomainError> {
        // 1. Validate and create value objects
        let email = Email::new(cmd.email)?;
        let password = PlainPassword::new(cmd.password)?;

        // 2. Check if user already exists
        if self.user_repo.exists_by_email(&email).await? {
            return Err(AuthDomainError::UserAlreadyExists);
        }

        // 3. Hash password
        let password_hash = self.password_hasher.hash(&password).await?;

        // 4. Create user
        let user_id = UserId::new(self.id_generator.generate());
        let user = User::new(user_id.clone(), email.clone(), password_hash);

        // 5. Save user
        self.user_repo.save(&user).await?;

        // 6. Generate tokens
        let token_pair = self
            .token_service
            .generate_token_pair(&user_id, email.as_str())
            .await?;

        // 7. Store refresh token
        self.store_refresh_token(&user_id, &token_pair.refresh_token, token_pair.refresh_expires_in)
            .await?;

        Ok(AuthResponse::new(
            user_id.into_inner(),
            email.into_inner(),
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.access_expires_in,
        ))
    }

    /// Login with email and password
    pub async fn login(&self, cmd: LoginCommand) -> Result<AuthResponse, AuthDomainError> {
        // 1. Validate email format
        let email = Email::new(cmd.email)?;
        let password = PlainPassword::new(cmd.password)?;

        // 2. Find user
        let user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or(AuthDomainError::InvalidCredentials)?;

        // 3. Verify password
        let password_hash = user
            .password_hash()
            .ok_or(AuthDomainError::InvalidCredentials)?;

        let is_valid = self
            .password_hasher
            .verify(&password, password_hash)
            .await?;

        if !is_valid {
            return Err(AuthDomainError::InvalidCredentials);
        }

        // 4. Generate tokens
        let token_pair = self
            .token_service
            .generate_token_pair(user.id(), user.email().as_str())
            .await?;

        // 5. Store refresh token
        self.store_refresh_token(user.id(), &token_pair.refresh_token, token_pair.refresh_expires_in)
            .await?;

        Ok(AuthResponse::new(
            user.id().as_str().to_string(),
            user.email().as_str().to_string(),
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.access_expires_in,
        ))
    }

    /// Logout - revoke refresh tokens
    pub async fn logout(&self, cmd: LogoutCommand) -> Result<(), AuthDomainError> {
        let user_id = UserId::new(cmd.user_id);

        if let Some(refresh_token) = cmd.refresh_token {
            // Revoke specific token only
            let token_hash = self.token_service.hash_refresh_token(&refresh_token);
            if let Some(token) = self.token_repo.find_by_hash(&token_hash).await? {
                self.token_repo.revoke(token.id()).await?;
            }
        } else {
            // Revoke all tokens for user (logout everywhere)
            self.token_repo.revoke_all_for_user(&user_id).await?;
        }

        Ok(())
    }

    /// Refresh access token using refresh token
    pub async fn refresh(&self, cmd: RefreshTokenCommand) -> Result<AuthResponse, AuthDomainError> {
        // 1. Decode and validate refresh token
        let claims = self
            .token_service
            .decode_refresh_token(&cmd.refresh_token)?;

        // 2. Find stored token by hash
        let token_hash = self.token_service.hash_refresh_token(&cmd.refresh_token);
        let stored_token = self
            .token_repo
            .find_by_hash(&token_hash)
            .await?
            .ok_or(AuthDomainError::TokenInvalid)?;

        // 3. Check if token is still valid
        if !stored_token.is_valid() {
            return Err(AuthDomainError::TokenRevoked);
        }

        // 4. Revoke old token (token rotation)
        self.token_repo.revoke(stored_token.id()).await?;

        // 5. Generate new token pair
        let user_id = UserId::new(claims.user_id);
        let token_pair = self
            .token_service
            .generate_token_pair(&user_id, &claims.email)
            .await?;

        // 6. Store new refresh token
        self.store_refresh_token(&user_id, &token_pair.refresh_token, token_pair.refresh_expires_in)
            .await?;

        Ok(AuthResponse::new(
            user_id.into_inner(),
            claims.email,
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.access_expires_in,
        ))
    }

    /// Get current user from token claims
    pub async fn get_current_user(&self, user_id: &str) -> Result<User, AuthDomainError> {
        let user_id = UserId::new(user_id.to_string());
        self.user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(AuthDomainError::UserNotFound)
    }

    /// Helper: store refresh token in database
    async fn store_refresh_token(
        &self,
        user_id: &UserId,
        refresh_token: &str,
        expires_in_secs: i64,
    ) -> Result<(), AuthDomainError> {
        let token_hash = self.token_service.hash_refresh_token(refresh_token);
        let token_id = TokenId::new(self.id_generator.generate());
        let expires_at = Utc::now() + Duration::seconds(expires_in_secs);

        let token = RefreshToken::new(token_id, user_id.clone(), token_hash, expires_at);

        self.token_repo.save(&token).await
    }
}
