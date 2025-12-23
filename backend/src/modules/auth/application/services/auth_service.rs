use std::sync::Arc;

use chrono::{Duration, Utc};
use rand::Rng;

use crate::modules::auth::application::dto::{
    AuthResponse, ChangeEmailCommand, ChangePasswordCommand, LoginCommand, LogoutCommand,
    RefreshTokenCommand, RegisterUserCommand,
};
use crate::modules::auth::application::ports::{IdGenerator, OrgContext, TokenService};
use crate::modules::auth::domain::{
    AuthDomainError, Email, PasswordHash, PasswordHasher, PlainPassword, RefreshToken,
    RefreshTokenRepository, TokenId, User, UserId, UserRepository,
};
use crate::modules::organizations::domain::{
    MemberId, OrgId, OrgName, OrgRole, OrgSlug, Organization, OrganizationMember,
    OrganizationMemberRepository, OrganizationRepository,
};

/// Authentication service - orchestrates all auth use cases
pub struct AuthService<U, T, P, TS, ID, OR, MR>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    user_repo: Arc<U>,
    token_repo: Arc<T>,
    password_hasher: Arc<P>,
    token_service: Arc<TS>,
    id_generator: Arc<ID>,
    org_repo: Arc<OR>,
    member_repo: Arc<MR>,
    /// Pre-computed dummy hash for timing attack mitigation
    dummy_password_hash: PasswordHash,
}

impl<U, T, P, TS, ID, OR, MR> AuthService<U, T, P, TS, ID, OR, MR>
where
    U: UserRepository,
    T: RefreshTokenRepository,
    P: PasswordHasher,
    TS: TokenService,
    ID: IdGenerator,
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
{
    pub fn new(
        user_repo: Arc<U>,
        token_repo: Arc<T>,
        password_hasher: Arc<P>,
        token_service: Arc<TS>,
        id_generator: Arc<ID>,
        org_repo: Arc<OR>,
        member_repo: Arc<MR>,
    ) -> Self {
        // Pre-computed Argon2 hash for timing attack mitigation
        // This ensures login takes consistent time whether user exists or not
        let dummy_password_hash = PasswordHash::from_hash(
            "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()
        );

        Self {
            user_repo,
            token_repo,
            password_hasher,
            token_service,
            id_generator,
            org_repo,
            member_repo,
            dummy_password_hash,
        }
    }

    /// Generate a random 4-character suffix for slugs
    fn generate_random_suffix(&self) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        (0..4)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Create a personal organization for a new user
    async fn create_personal_org_for_user(
        &self,
        user_id: &UserId,
        email: &str,
    ) -> Result<(OrgId, OrgRole), AuthDomainError> {
        // Extract name from email prefix
        let email_prefix = email.split('@').next().unwrap_or("user");
        let name = OrgName::new(email_prefix.to_string())
            .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        // Generate slug with random suffix
        let suffix = self.generate_random_suffix();
        let slug = OrgSlug::generate(&name, &suffix);

        // Create personal organization
        let org_id = OrgId::new(self.id_generator.generate());
        let org = Organization::new_personal(org_id.clone(), name, slug);

        // Save organization
        self.org_repo
            .save(&org)
            .await
            .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        // Create membership as owner with last_accessed set
        let member_id = MemberId::new(self.id_generator.generate());
        let mut member = OrganizationMember::new(
            member_id,
            org_id.clone(),
            user_id.clone(),
            OrgRole::Owner,
        );
        member.touch_last_accessed();
        self.member_repo
            .save(&member)
            .await
            .map_err(|e| AuthDomainError::InternalError(e.to_string()))?;

        Ok((org_id, OrgRole::Owner))
    }

    /// Get the default organization for a user (last accessed or personal)
    async fn get_default_org_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Option<(OrgId, OrgRole)>, AuthDomainError> {
        // Try last accessed first
        if let Some(membership) = self
            .member_repo
            .find_last_accessed_by_user(user_id)
            .await
            .map_err(|e| AuthDomainError::InternalError(e.to_string()))?
        {
            // Verify org still exists and not deleted
            if let Some(org) = self
                .org_repo
                .find_by_id(membership.organization_id())
                .await
                .map_err(|e| AuthDomainError::InternalError(e.to_string()))?
            {
                if !org.is_deleted() {
                    return Ok(Some((
                        OrgId::new(org.id().as_str().to_string()),
                        *membership.role(),
                    )));
                }
            }
        }

        // Fall back to personal org
        if let Some(membership) = self
            .member_repo
            .find_personal_org_membership(user_id)
            .await
            .map_err(|e| AuthDomainError::InternalError(e.to_string()))?
        {
            return Ok(Some((
                OrgId::new(membership.organization_id().as_str().to_string()),
                *membership.role(),
            )));
        }

        Ok(None)
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

        // 6. Create personal organization for the new user
        let (org_id, org_role) = self
            .create_personal_org_for_user(&user_id, email.as_str())
            .await?;

        // 7. Generate tokens with org context
        let org_context = Some(OrgContext {
            org_id: org_id.as_str().to_string(),
            org_role: org_role.as_str().to_string(),
        });
        let token_pair = self
            .token_service
            .generate_token_pair(&user_id, email.as_str(), org_context)
            .await?;

        // 8. Store refresh token with device fingerprint
        self.store_refresh_token(
            &user_id,
            &token_pair.refresh_token,
            &cmd.device_fingerprint,
            token_pair.refresh_expires_in,
        )
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
        // Use for_verification to skip strength validation - we only need to compare against hash
        let password = PlainPassword::for_verification(cmd.password);

        // 2. Find user (don't early-return to prevent timing attacks)
        let user_opt = self.user_repo.find_by_email(&email).await?;

        // 3. Get password hash or use dummy for timing consistency
        let (user, password_hash) = match &user_opt {
            Some(user) => {
                let hash = user
                    .password_hash()
                    .cloned()
                    .unwrap_or_else(|| self.dummy_password_hash.clone());
                (Some(user), hash)
            }
            None => (None, self.dummy_password_hash.clone()),
        };

        // 4. Always verify password (timing-safe: takes same time regardless of user existence)
        let is_valid = self
            .password_hasher
            .verify(&password, &password_hash)
            .await?;

        // 5. Check both user existence AND password validity
        let user = match (user, is_valid) {
            (Some(u), true) => u,
            _ => return Err(AuthDomainError::InvalidCredentials),
        };

        // 6. Get default organization for user (last accessed or personal)
        let org_context = if let Some((org_id, org_role)) =
            self.get_default_org_for_user(user.id()).await?
        {
            Some(OrgContext {
                org_id: org_id.as_str().to_string(),
                org_role: org_role.as_str().to_string(),
            })
        } else {
            None
        };

        // 7. Generate tokens with org context
        let token_pair = self
            .token_service
            .generate_token_pair(user.id(), user.email().as_str(), org_context)
            .await?;

        // 8. Store refresh token with device fingerprint
        self.store_refresh_token(
            user.id(),
            &token_pair.refresh_token,
            &cmd.device_fingerprint,
            token_pair.refresh_expires_in,
        )
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

        // 4. Validate device fingerprint matches
        if !stored_token.matches_fingerprint(&cmd.device_fingerprint) {
            // Token used from different device - suspicious activity, revoke it
            self.token_repo.revoke(stored_token.id()).await?;
            return Err(AuthDomainError::TokenInvalid);
        }

        // 5. Revoke old token (token rotation)
        self.token_repo.revoke(stored_token.id()).await?;

        // 6. Re-query default organization for user (last accessed or personal)
        // This ensures tokens reflect current org context even if user switched orgs
        let user_id = UserId::new(claims.user_id);
        let org_context = if let Some((org_id, org_role)) =
            self.get_default_org_for_user(&user_id).await?
        {
            Some(OrgContext {
                org_id: org_id.as_str().to_string(),
                org_role: org_role.as_str().to_string(),
            })
        } else {
            None
        };

        // 7. Generate new token pair with org context
        let token_pair = self
            .token_service
            .generate_token_pair(&user_id, &claims.email, org_context)
            .await?;

        // 8. Store new refresh token with same device fingerprint
        self.store_refresh_token(
            &user_id,
            &token_pair.refresh_token,
            &cmd.device_fingerprint,
            token_pair.refresh_expires_in,
        )
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

    /// Change user's email address
    pub async fn change_email(&self, cmd: ChangeEmailCommand) -> Result<(), AuthDomainError> {
        // 1. Validate new email format
        let new_email = Email::new(cmd.new_email)?;

        // 2. Get current user
        let user_id = UserId::new(cmd.user_id);
        let mut user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(AuthDomainError::UserNotFound)?;

        // 3. Check user has a password (not OAuth-only)
        let password_hash = user
            .password_hash()
            .cloned()
            .ok_or(AuthDomainError::NoPasswordSet)?;

        // 4. Verify current password
        let current_password = PlainPassword::for_verification(cmd.current_password);
        let is_valid = self
            .password_hasher
            .verify(&current_password, &password_hash)
            .await?;
        if !is_valid {
            return Err(AuthDomainError::InvalidCredentials);
        }

        // 5. Check if new email is already in use (by another user)
        if self.user_repo.exists_by_email(&new_email).await? {
            // If same email (case normalization), no change needed
            if user.email().as_str() != new_email.as_str() {
                return Err(AuthDomainError::EmailAlreadyInUse);
            }
            return Ok(());
        }

        // 6. Update email
        user.update_email(new_email);

        // 7. Persist changes
        self.user_repo.save(&user).await
    }

    /// Change user's password
    pub async fn change_password(&self, cmd: ChangePasswordCommand) -> Result<(), AuthDomainError> {
        // 1. Validate new password strength
        let new_password = PlainPassword::new(cmd.new_password)?;

        // 2. Get current user
        let user_id = UserId::new(cmd.user_id);
        let mut user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or(AuthDomainError::UserNotFound)?;

        // 3. Check user has a password (not OAuth-only)
        let password_hash = user
            .password_hash()
            .cloned()
            .ok_or(AuthDomainError::NoPasswordSet)?;

        // 4. Verify current password
        let current_password = PlainPassword::for_verification(cmd.current_password);
        let is_valid = self
            .password_hasher
            .verify(&current_password, &password_hash)
            .await?;
        if !is_valid {
            return Err(AuthDomainError::InvalidCredentials);
        }

        // 5. Hash new password
        let new_password_hash = self.password_hasher.hash(&new_password).await?;

        // 6. Update password
        user.update_password(new_password_hash);

        // 7. Persist changes
        self.user_repo.save(&user).await
    }

    /// Helper: store refresh token in database
    async fn store_refresh_token(
        &self,
        user_id: &UserId,
        refresh_token: &str,
        device_fingerprint: &str,
        expires_in_secs: i64,
    ) -> Result<(), AuthDomainError> {
        let token_hash = self.token_service.hash_refresh_token(refresh_token);
        let token_id = TokenId::new(self.id_generator.generate());
        let expires_at = Utc::now() + Duration::seconds(expires_in_secs);

        let token = RefreshToken::new(
            token_id,
            user_id.clone(),
            token_hash,
            device_fingerprint.to_string(),
            expires_at,
        );

        self.token_repo.save(&token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::auth::application::ports::{TokenClaims, TokenPair};
    use crate::modules::auth::domain::PasswordHasher;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ==================== Mock Implementations ====================

    /// Mock User Repository
    struct MockUserRepository {
        users: Mutex<HashMap<String, User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }

        fn with_user(user: User) -> Self {
            let repo = Self::new();
            repo.users.lock().unwrap().insert(user.email().as_str().to_string(), user);
            repo
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, AuthDomainError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.id().as_str() == id.as_str()).cloned())
        }

        async fn find_by_email(&self, email: &Email) -> Result<Option<User>, AuthDomainError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(email.as_str()).cloned())
        }

        async fn save(&self, user: &User) -> Result<(), AuthDomainError> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.email().as_str().to_string(), user.clone());
            Ok(())
        }

        async fn exists_by_email(&self, email: &Email) -> Result<bool, AuthDomainError> {
            let users = self.users.lock().unwrap();
            Ok(users.contains_key(email.as_str()))
        }
    }

    /// Mock Refresh Token Repository
    struct MockRefreshTokenRepository {
        tokens: Mutex<HashMap<String, RefreshToken>>,
    }

    impl MockRefreshTokenRepository {
        fn new() -> Self {
            Self {
                tokens: Mutex::new(HashMap::new()),
            }
        }

        fn with_token(token: RefreshToken) -> Self {
            let repo = Self::new();
            repo.tokens.lock().unwrap().insert(token.token_hash().to_string(), token);
            repo
        }
    }

    #[async_trait::async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
        async fn save(&self, token: &RefreshToken) -> Result<(), AuthDomainError> {
            let mut tokens = self.tokens.lock().unwrap();
            tokens.insert(token.token_hash().to_string(), token.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &TokenId) -> Result<Option<RefreshToken>, AuthDomainError> {
            let tokens = self.tokens.lock().unwrap();
            Ok(tokens.values().find(|t| t.id().as_str() == id.as_str()).cloned())
        }

        async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthDomainError> {
            let tokens = self.tokens.lock().unwrap();
            Ok(tokens.get(hash).cloned())
        }

        async fn revoke(&self, id: &TokenId) -> Result<(), AuthDomainError> {
            let mut tokens = self.tokens.lock().unwrap();
            if let Some(token) = tokens.values_mut().find(|t| t.id().as_str() == id.as_str()) {
                token.revoke();
            }
            Ok(())
        }

        async fn revoke_all_for_user(&self, user_id: &UserId) -> Result<(), AuthDomainError> {
            let mut tokens = self.tokens.lock().unwrap();
            for token in tokens.values_mut() {
                if token.user_id().as_str() == user_id.as_str() {
                    token.revoke();
                }
            }
            Ok(())
        }

        async fn delete_expired(&self) -> Result<u64, AuthDomainError> {
            let mut tokens = self.tokens.lock().unwrap();
            let before = tokens.len();
            tokens.retain(|_, t| !t.is_expired());
            Ok((before - tokens.len()) as u64)
        }
    }

    /// Mock Password Hasher
    struct MockPasswordHasher;

    #[async_trait::async_trait]
    impl PasswordHasher for MockPasswordHasher {
        async fn hash(&self, password: &PlainPassword) -> Result<PasswordHash, AuthDomainError> {
            Ok(PasswordHash::from_hash(format!("hashed_{}", password.as_str())))
        }

        async fn verify(
            &self,
            password: &PlainPassword,
            hash: &PasswordHash,
        ) -> Result<bool, AuthDomainError> {
            Ok(hash.as_str() == format!("hashed_{}", password.as_str()))
        }
    }

    /// Mock Token Service
    struct MockTokenService {
        should_fail_decode: bool,
    }

    impl MockTokenService {
        fn new() -> Self {
            Self { should_fail_decode: false }
        }

        fn failing_decode() -> Self {
            Self { should_fail_decode: true }
        }
    }

    #[async_trait::async_trait]
    impl TokenService for MockTokenService {
        async fn generate_token_pair(
            &self,
            user_id: &UserId,
            _email: &str,
            _org_context: Option<crate::modules::auth::application::ports::OrgContext>,
        ) -> Result<TokenPair, AuthDomainError> {
            Ok(TokenPair {
                access_token: format!("access_token_{}", user_id.as_str()),
                refresh_token: format!("refresh_token_{}", user_id.as_str()),
                access_expires_in: 900,
                refresh_expires_in: 604800,
            })
        }

        fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthDomainError> {
            if token.starts_with("access_token_") {
                let user_id = token.replace("access_token_", "");
                Ok(TokenClaims {
                    user_id,
                    email: "test@example.com".to_string(),
                    org_id: None,
                    org_role: None,
                    exp: Utc::now().timestamp() + 900,
                    iat: Utc::now().timestamp(),
                })
            } else {
                Err(AuthDomainError::TokenInvalid)
            }
        }

        fn decode_refresh_token(&self, token: &str) -> Result<TokenClaims, AuthDomainError> {
            if self.should_fail_decode {
                return Err(AuthDomainError::TokenInvalid);
            }
            if token.starts_with("refresh_token_") {
                let user_id = token.replace("refresh_token_", "");
                Ok(TokenClaims {
                    user_id,
                    email: "test@example.com".to_string(),
                    org_id: None,
                    org_role: None,
                    exp: Utc::now().timestamp() + 604800,
                    iat: Utc::now().timestamp(),
                })
            } else {
                Err(AuthDomainError::TokenInvalid)
            }
        }

        fn hash_refresh_token(&self, token: &str) -> String {
            format!("hash_{}", token)
        }
    }

    /// Mock ID Generator
    struct MockIdGenerator {
        counter: Mutex<u32>,
    }

    impl MockIdGenerator {
        fn new() -> Self {
            Self { counter: Mutex::new(0) }
        }
    }

    impl IdGenerator for MockIdGenerator {
        fn generate(&self) -> String {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            format!("generated-id-{}", counter)
        }
    }

    /// Mock Organization Repository
    struct MockOrganizationRepository {
        orgs: Mutex<HashMap<String, Organization>>,
    }

    impl MockOrganizationRepository {
        fn new() -> Self {
            Self {
                orgs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl OrganizationRepository for MockOrganizationRepository {
        async fn find_by_id(
            &self,
            id: &OrgId,
        ) -> Result<Option<Organization>, crate::modules::organizations::domain::OrgDomainError> {
            let orgs = self.orgs.lock().unwrap();
            Ok(orgs.get(id.as_str()).cloned())
        }

        async fn find_by_slug(
            &self,
            slug: &str,
        ) -> Result<Option<Organization>, crate::modules::organizations::domain::OrgDomainError> {
            let orgs = self.orgs.lock().unwrap();
            Ok(orgs.values().find(|o| o.slug().as_str() == slug).cloned())
        }

        async fn save(
            &self,
            org: &Organization,
        ) -> Result<(), crate::modules::organizations::domain::OrgDomainError> {
            let mut orgs = self.orgs.lock().unwrap();
            orgs.insert(org.id().as_str().to_string(), org.clone());
            Ok(())
        }

        async fn slug_exists(
            &self,
            slug: &str,
        ) -> Result<bool, crate::modules::organizations::domain::OrgDomainError> {
            let orgs = self.orgs.lock().unwrap();
            Ok(orgs.values().any(|o| o.slug().as_str() == slug))
        }
    }

    /// Mock Organization Member Repository
    struct MockOrganizationMemberRepository {
        members: Mutex<HashMap<String, OrganizationMember>>,
    }

    impl MockOrganizationMemberRepository {
        fn new() -> Self {
            Self {
                members: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl OrganizationMemberRepository for MockOrganizationMemberRepository {
        async fn find_by_id(
            &self,
            id: &MemberId,
        ) -> Result<Option<OrganizationMember>, crate::modules::organizations::domain::OrgDomainError>
        {
            let members = self.members.lock().unwrap();
            Ok(members.get(id.as_str()).cloned())
        }

        async fn find_by_org_and_user(
            &self,
            org_id: &OrgId,
            user_id: &UserId,
        ) -> Result<Option<OrganizationMember>, crate::modules::organizations::domain::OrgDomainError>
        {
            let members = self.members.lock().unwrap();
            Ok(members
                .values()
                .find(|m| {
                    m.organization_id().as_str() == org_id.as_str()
                        && m.user_id().as_str() == user_id.as_str()
                })
                .cloned())
        }

        async fn find_all_by_org(
            &self,
            org_id: &OrgId,
        ) -> Result<Vec<OrganizationMember>, crate::modules::organizations::domain::OrgDomainError>
        {
            let members = self.members.lock().unwrap();
            Ok(members
                .values()
                .filter(|m| m.organization_id().as_str() == org_id.as_str())
                .cloned()
                .collect())
        }

        async fn find_all_by_user(
            &self,
            user_id: &UserId,
        ) -> Result<Vec<OrganizationMember>, crate::modules::organizations::domain::OrgDomainError>
        {
            let members = self.members.lock().unwrap();
            Ok(members
                .values()
                .filter(|m| m.user_id().as_str() == user_id.as_str())
                .cloned()
                .collect())
        }

        async fn find_last_accessed_by_user(
            &self,
            user_id: &UserId,
        ) -> Result<Option<OrganizationMember>, crate::modules::organizations::domain::OrgDomainError>
        {
            let members = self.members.lock().unwrap();
            Ok(members
                .values()
                .filter(|m| m.user_id().as_str() == user_id.as_str() && m.last_accessed_at().is_some())
                .max_by_key(|m| m.last_accessed_at())
                .cloned())
        }

        async fn find_personal_org_membership(
            &self,
            _user_id: &UserId,
        ) -> Result<Option<OrganizationMember>, crate::modules::organizations::domain::OrgDomainError>
        {
            // For tests, we don't track personal orgs, just return None
            Ok(None)
        }

        async fn save(
            &self,
            member: &OrganizationMember,
        ) -> Result<(), crate::modules::organizations::domain::OrgDomainError> {
            let mut members = self.members.lock().unwrap();
            members.insert(member.id().as_str().to_string(), member.clone());
            Ok(())
        }

        async fn delete(
            &self,
            id: &MemberId,
        ) -> Result<(), crate::modules::organizations::domain::OrgDomainError> {
            let mut members = self.members.lock().unwrap();
            members.remove(id.as_str());
            Ok(())
        }

        async fn count_owners(
            &self,
            org_id: &OrgId,
        ) -> Result<u32, crate::modules::organizations::domain::OrgDomainError> {
            let members = self.members.lock().unwrap();
            Ok(members
                .values()
                .filter(|m| {
                    m.organization_id().as_str() == org_id.as_str()
                        && m.role() == &OrgRole::Owner
                })
                .count() as u32)
        }

        async fn count_owners_for_update(
            &self,
            org_id: &OrgId,
        ) -> Result<u32, crate::modules::organizations::domain::OrgDomainError> {
            self.count_owners(org_id).await
        }
    }

    // ==================== Test Helpers ====================

    fn create_auth_service() -> AuthService<
        MockUserRepository,
        MockRefreshTokenRepository,
        MockPasswordHasher,
        MockTokenService,
        MockIdGenerator,
        MockOrganizationRepository,
        MockOrganizationMemberRepository,
    > {
        AuthService::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(MockRefreshTokenRepository::new()),
            Arc::new(MockPasswordHasher),
            Arc::new(MockTokenService::new()),
            Arc::new(MockIdGenerator::new()),
            Arc::new(MockOrganizationRepository::new()),
            Arc::new(MockOrganizationMemberRepository::new()),
        )
    }

    fn create_auth_service_with_user(
        user: User,
    ) -> AuthService<
        MockUserRepository,
        MockRefreshTokenRepository,
        MockPasswordHasher,
        MockTokenService,
        MockIdGenerator,
        MockOrganizationRepository,
        MockOrganizationMemberRepository,
    > {
        AuthService::new(
            Arc::new(MockUserRepository::with_user(user)),
            Arc::new(MockRefreshTokenRepository::new()),
            Arc::new(MockPasswordHasher),
            Arc::new(MockTokenService::new()),
            Arc::new(MockIdGenerator::new()),
            Arc::new(MockOrganizationRepository::new()),
            Arc::new(MockOrganizationMemberRepository::new()),
        )
    }

    fn create_test_user(id: &str, email: &str, password: &str) -> User {
        User::new(
            UserId::new(id.to_string()),
            Email::new(email.to_string()).unwrap(),
            PasswordHash::from_hash(format!("hashed_{}", password)),
        )
    }

    // ==================== Registration Tests ====================

    #[tokio::test]
    async fn test_register_success() {
        let service = create_auth_service();

        let cmd = RegisterUserCommand {
            email: "newuser@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.register(cmd).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.email, "newuser@example.com");
        assert!(!response.access_token.is_empty());
        assert!(!response.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let existing_user = create_test_user("user-1", "existing@example.com", "password123");
        let service = create_auth_service_with_user(existing_user);

        let cmd = RegisterUserCommand {
            email: "existing@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.register(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::UserAlreadyExists)));
    }

    #[tokio::test]
    async fn test_register_invalid_email() {
        let service = create_auth_service();

        let cmd = RegisterUserCommand {
            email: "invalid-email".to_string(),
            password: "SecurePass123!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.register(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::InvalidEmail(_))));
    }

    #[tokio::test]
    async fn test_register_weak_password() {
        let service = create_auth_service();

        let cmd = RegisterUserCommand {
            email: "user@example.com".to_string(),
            password: "short".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.register(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::WeakPassword(_))));
    }

    // ==================== Login Tests ====================

    #[tokio::test]
    async fn test_login_success() {
        let user = create_test_user("user-1", "test@example.com", "CorrectPass1!");
        let service = create_auth_service_with_user(user);

        let cmd = LoginCommand {
            email: "test@example.com".to_string(),
            password: "CorrectPass1!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.login(cmd).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.user_id, "user-1");
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let user = create_test_user("user-1", "test@example.com", "CorrectPass1!");
        let service = create_auth_service_with_user(user);

        let cmd = LoginCommand {
            email: "test@example.com".to_string(),
            password: "WrongPass1!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.login(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let service = create_auth_service();

        let cmd = LoginCommand {
            email: "nonexistent@example.com".to_string(),
            password: "SomePass1!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.login(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_invalid_email_format() {
        let service = create_auth_service();

        let cmd = LoginCommand {
            email: "not-an-email".to_string(),
            password: "SomePass1!".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.login(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::InvalidEmail(_))));
    }

    // ==================== Logout Tests ====================

    #[tokio::test]
    async fn test_logout_single_token() {
        let user = create_test_user("user-1", "test@example.com", "password");
        let token = RefreshToken::new(
            TokenId::new("token-1".to_string()),
            UserId::new("user-1".to_string()),
            "hash_refresh_token_user-1".to_string(),
            "test-fingerprint".to_string(),
            Utc::now() + Duration::days(7),
        );

        let service = AuthService::new(
            Arc::new(MockUserRepository::with_user(user)),
            Arc::new(MockRefreshTokenRepository::with_token(token)),
            Arc::new(MockPasswordHasher),
            Arc::new(MockTokenService::new()),
            Arc::new(MockIdGenerator::new()),
            Arc::new(MockOrganizationRepository::new()),
            Arc::new(MockOrganizationMemberRepository::new()),
        );

        let cmd = LogoutCommand {
            user_id: "user-1".to_string(),
            refresh_token: Some("refresh_token_user-1".to_string()),
        };

        let result = service.logout(cmd).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logout_all_tokens() {
        let service = create_auth_service();

        let cmd = LogoutCommand {
            user_id: "user-1".to_string(),
            refresh_token: None,
        };

        let result = service.logout(cmd).await;

        assert!(result.is_ok());
    }

    // ==================== Refresh Token Tests ====================

    #[tokio::test]
    async fn test_refresh_success() {
        let user_id = UserId::new("user-1".to_string());
        let token = RefreshToken::new(
            TokenId::new("token-1".to_string()),
            user_id.clone(),
            "hash_refresh_token_user-1".to_string(),
            "test-fingerprint".to_string(),
            Utc::now() + Duration::days(7),
        );

        let user = create_test_user("user-1", "test@example.com", "password");

        let service = AuthService::new(
            Arc::new(MockUserRepository::with_user(user)),
            Arc::new(MockRefreshTokenRepository::with_token(token)),
            Arc::new(MockPasswordHasher),
            Arc::new(MockTokenService::new()),
            Arc::new(MockIdGenerator::new()),
            Arc::new(MockOrganizationRepository::new()),
            Arc::new(MockOrganizationMemberRepository::new()),
        );

        let cmd = RefreshTokenCommand {
            refresh_token: "refresh_token_user-1".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.refresh(cmd).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.user_id, "user-1");
    }

    #[tokio::test]
    async fn test_refresh_invalid_token() {
        let service = AuthService::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(MockRefreshTokenRepository::new()),
            Arc::new(MockPasswordHasher),
            Arc::new(MockTokenService::failing_decode()),
            Arc::new(MockIdGenerator::new()),
            Arc::new(MockOrganizationRepository::new()),
            Arc::new(MockOrganizationMemberRepository::new()),
        );

        let cmd = RefreshTokenCommand {
            refresh_token: "invalid_token".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.refresh(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::TokenInvalid)));
    }

    #[tokio::test]
    async fn test_refresh_token_not_in_database() {
        let service = create_auth_service();

        let cmd = RefreshTokenCommand {
            refresh_token: "refresh_token_user-1".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.refresh(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::TokenInvalid)));
    }

    #[tokio::test]
    async fn test_refresh_revoked_token() {
        let user_id = UserId::new("user-1".to_string());
        let mut token = RefreshToken::new(
            TokenId::new("token-1".to_string()),
            user_id.clone(),
            "hash_refresh_token_user-1".to_string(),
            "test-fingerprint".to_string(),
            Utc::now() + Duration::days(7),
        );
        token.revoke();

        let service = AuthService::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(MockRefreshTokenRepository::with_token(token)),
            Arc::new(MockPasswordHasher),
            Arc::new(MockTokenService::new()),
            Arc::new(MockIdGenerator::new()),
            Arc::new(MockOrganizationRepository::new()),
            Arc::new(MockOrganizationMemberRepository::new()),
        );

        let cmd = RefreshTokenCommand {
            refresh_token: "refresh_token_user-1".to_string(),
            device_fingerprint: "test-fingerprint".to_string(),
        };

        let result = service.refresh(cmd).await;

        assert!(matches!(result, Err(AuthDomainError::TokenRevoked)));
    }

    // ==================== Get Current User Tests ====================

    #[tokio::test]
    async fn test_get_current_user_success() {
        let user = create_test_user("user-1", "test@example.com", "password");
        let service = create_auth_service_with_user(user);

        let result = service.get_current_user("user-1").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id().as_str(), "user-1");
        assert_eq!(user.email().as_str(), "test@example.com");
    }

    #[tokio::test]
    async fn test_get_current_user_not_found() {
        let service = create_auth_service();

        let result = service.get_current_user("nonexistent-user").await;

        assert!(matches!(result, Err(AuthDomainError::UserNotFound)));
    }
}
