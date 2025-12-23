mod config;
mod modules;

use std::sync::Arc;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::modules::auth::{
    application::AuthService,
    domain::RefreshTokenRepository,
    infrastructure::{
        Argon2PasswordHasher, IpRateLimiter, JwtConfig, JwtTokenService,
        PostgresRefreshTokenRepository, PostgresUserRepository, UuidGenerator, auth_routes,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,sqlx=warn".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting server on {}", config.addr());

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    let pool = Arc::new(pool);

    tracing::info!("Connected to database");

    // Create infrastructure services
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let jwt_config = JwtConfig::new(
        config.jwt_access_secret.clone(),
        config.jwt_refresh_secret.clone(),
        15 * 60, // 15 minutes for access token
        config.refresh_token_duration_days * 24 * 60 * 60, // days to seconds
    );
    let token_service = Arc::new(JwtTokenService::new(jwt_config));
    let id_generator = Arc::new(UuidGenerator::new());

    // Spawn background task for token cleanup
    {
        let cleanup_repo = token_repo.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60)); // 1 hour
            loop {
                interval.tick().await;
                match cleanup_repo.delete_expired().await {
                    Ok(count) if count > 0 => {
                        tracing::info!(deleted_count = count, "Cleaned up expired refresh tokens");
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to clean up expired refresh tokens");
                    }
                    _ => {}
                }
            }
        });
        tracing::info!("Token cleanup task started (runs every hour)");
    }

    // Create auth service
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        token_repo,
        password_hasher,
        token_service.clone(),
        id_generator,
    ));

    // Create rate limiter (10 requests per minute per IP)
    let rate_limiter = Arc::new(IpRateLimiter::new(10));

    // Spawn background task for rate limiter cleanup
    {
        let cleanup_limiter = rate_limiter.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5 * 60)); // 5 minutes
            loop {
                interval.tick().await;
                cleanup_limiter.cleanup_if_needed(10_000).await;
            }
        });
        tracing::info!("Rate limiter cleanup task started (runs every 5 minutes, max 10k entries)");
    }

    // Create router
    let app = Router::new()
        .nest("/api/auth", auth_routes(auth_service, token_service, rate_limiter))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(config.addr()).await?;
    tracing::info!("Server listening on {}", config.addr());

    axum::serve(listener, app).await?;

    Ok(())
}
