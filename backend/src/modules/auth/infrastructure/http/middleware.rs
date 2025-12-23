use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use super::extractors::AuthClaims;
use crate::modules::auth::application::TokenService;

/// Authentication middleware - validates JWT and injects AuthClaims into request
pub async fn auth_middleware<TS: TokenService + 'static>(
    State(token_service): State<Arc<TS>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check Bearer prefix
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let claims = token_service
        .validate_access_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insert claims into request extensions
    req.extensions_mut().insert(AuthClaims {
        user_id: claims.user_id,
        email: claims.email,
    });

    Ok(next.run(req).await)
}
