use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::handlers;
use super::middleware::auth_middleware;
use super::rate_limit::{rate_limit_middleware, IpRateLimiter};
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::application::AuthService;
use crate::modules::auth::domain::{PasswordHasher, RefreshTokenRepository, UserRepository};
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};

/// Create auth routes
pub fn auth_routes<U, T, P, TS, ID, OR, MR>(
    auth_service: Arc<AuthService<U, T, P, TS, ID, OR, MR>>,
    token_service: Arc<TS>,
    rate_limiter: Arc<IpRateLimiter>,
) -> Router
where
    U: UserRepository + 'static,
    T: RefreshTokenRepository + 'static,
    P: PasswordHasher + 'static,
    TS: TokenService + 'static,
    ID: IdGenerator + 'static,
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
{
    // Public routes with rate limiting
    let public_routes = Router::new()
        .route("/register", post(handlers::register::<U, T, P, TS, ID, OR, MR>))
        .route("/login", post(handlers::login::<U, T, P, TS, ID, OR, MR>))
        .route("/refresh", post(handlers::refresh::<U, T, P, TS, ID, OR, MR>))
        .layer(middleware::from_fn(move |req, next| {
            let limiter = rate_limiter.clone();
            async move { rate_limit_middleware(limiter, req, next).await }
        }));

    // Protected routes (require authentication, no rate limiting needed)
    let protected_routes = Router::new()
        .route("/logout", post(handlers::logout::<U, T, P, TS, ID, OR, MR>))
        .route("/me", get(handlers::me::<U, T, P, TS, ID, OR, MR>))
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(auth_service)
}
