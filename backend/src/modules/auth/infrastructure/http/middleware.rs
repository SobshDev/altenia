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

/// Extract token from query string (for SSE which doesn't support headers)
fn extract_token_from_query(uri: &axum::http::Uri) -> Option<String> {
    uri.query().and_then(|query| {
        query.split('&')
            .find_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next()?;
                if key == "token" {
                    Some(value.to_string())
                } else {
                    None
                }
            })
    })
}

/// Authentication middleware - validates JWT and injects AuthClaims into request
/// Supports both Authorization header and query parameter (for SSE endpoints)
pub async fn auth_middleware<TS: TokenService + 'static>(
    State(token_service): State<Arc<TS>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to extract token from Authorization header first
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(|t| t.to_string())
        // Fall back to query parameter (for SSE endpoints)
        .or_else(|| extract_token_from_query(req.uri()))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let claims = token_service
        .validate_access_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insert claims into request extensions
    req.extensions_mut().insert(AuthClaims {
        user_id: claims.user_id,
        email: claims.email,
        org_id: claims.org_id,
        org_role: claims.org_role,
    });

    Ok(next.run(req).await)
}
