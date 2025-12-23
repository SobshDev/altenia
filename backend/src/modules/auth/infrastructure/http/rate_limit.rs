use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::{
    collections::HashMap,
    net::IpAddr,
    num::NonZeroU32,
    sync::Arc,
};
use tokio::sync::RwLock;

/// Per-IP rate limiter
pub struct IpRateLimiter {
    limiters: RwLock<HashMap<IpAddr, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    quota: Quota,
}

impl IpRateLimiter {
    /// Create a new rate limiter with specified requests per minute
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap());
        Self {
            limiters: RwLock::new(HashMap::new()),
            quota,
        }
    }

    async fn get_or_create_limiter(
        &self,
        ip: IpAddr,
    ) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
        // Try read lock first
        {
            let limiters = self.limiters.read().await;
            if let Some(limiter) = limiters.get(&ip) {
                return limiter.clone();
            }
        }

        // Need write lock to insert
        let mut limiters = self.limiters.write().await;
        // Double-check after acquiring write lock
        if let Some(limiter) = limiters.get(&ip) {
            return limiter.clone();
        }

        let limiter = Arc::new(RateLimiter::direct(self.quota));
        limiters.insert(ip, limiter.clone());
        limiter
    }

    /// Check if the given IP is within rate limits
    pub async fn check(&self, ip: IpAddr) -> bool {
        let limiter = self.get_or_create_limiter(ip).await;
        limiter.check().is_ok()
    }

    /// Remove all entries when cache exceeds max size to prevent memory exhaustion
    /// Should be called periodically from a background task
    pub async fn cleanup_if_needed(&self, max_entries: usize) {
        let should_clear = {
            let limiters = self.limiters.read().await;
            limiters.len() > max_entries
        };

        if should_clear {
            let mut limiters = self.limiters.write().await;
            // Double-check after acquiring write lock
            if limiters.len() > max_entries {
                let old_size = limiters.len();
                limiters.clear();
                tracing::info!(
                    old_size = old_size,
                    max_entries = max_entries,
                    "Rate limiter cache cleared to prevent memory exhaustion"
                );
            }
        }
    }
}

/// Rate limiting middleware for axum
pub async fn rate_limit_middleware(
    rate_limiter: Arc<IpRateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Extract IP from connection info or X-Forwarded-For header
    let ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .or_else(|| {
            req.headers()
                .get("X-Forwarded-For")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or_else(|| "127.0.0.1".parse().unwrap());

    if !rate_limiter.check(ip).await {
        tracing::warn!(ip = %ip, "Rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();
    }

    next.run(req).await
}
