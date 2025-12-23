use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::handlers;
use super::middleware::auth_middleware;
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::application::AuthService;
use crate::modules::auth::domain::{PasswordHasher, RefreshTokenRepository, UserRepository};

/// Create auth routes
pub fn auth_routes<U, T, P, TS, ID>(
    auth_service: Arc<AuthService<U, T, P, TS, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    U: UserRepository + 'static,
    T: RefreshTokenRepository + 'static,
    P: PasswordHasher + 'static,
    TS: TokenService + 'static,
    ID: IdGenerator + 'static,
{
    // Public routes
    let public_routes = Router::new()
        .route("/register", post(handlers::register::<U, T, P, TS, ID>))
        .route("/login", post(handlers::login::<U, T, P, TS, ID>))
        .route("/refresh", post(handlers::refresh::<U, T, P, TS, ID>));

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/logout", post(handlers::logout::<U, T, P, TS, ID>))
        .route("/me", get(handlers::me::<U, T, P, TS, ID>))
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(auth_service)
}
