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
    infrastructure::{
        Argon2PasswordHasher, JwtConfig, JwtTokenService, PostgresRefreshTokenRepository,
        PostgresUserRepository, UuidGenerator, auth_routes,
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
    let jwt_config = JwtConfig::default_with_secrets(
        config.jwt_access_secret.clone(),
        config.jwt_refresh_secret.clone(),
    );
    let token_service = Arc::new(JwtTokenService::new(jwt_config));
    let id_generator = Arc::new(UuidGenerator::new());

    // Create auth service
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        token_repo,
        password_hasher,
        token_service.clone(),
        id_generator,
    ));

    // Create router
    let app = Router::new()
        .nest("/api/auth", auth_routes(auth_service, token_service))
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
