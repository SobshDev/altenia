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
use crate::modules::organizations::{
    application::services::{InviteService, OrgService},
    domain::OrganizationInviteRepository,
    infrastructure::{
        org_invite_routes, org_routes, user_invite_routes, PostgresInviteRepository,
        PostgresOrgActivityRepository, PostgresOrganizationMemberRepository,
        PostgresOrganizationRepository,
    },
};
use crate::modules::projects::{
    application::ProjectService,
    infrastructure::{project_routes, PostgresApiKeyRepository, PostgresProjectRepository},
};
use crate::modules::logging::{
    application::services::{FilterPresetService, LogService},
    infrastructure::{
        filter_preset_routes, ingest_routes, log_query_routes, sse_routes, start_cleanup_task,
        start_log_listener, LogBroadcaster, PostgresFilterPresetRepository, TimescaleLogRepository,
    },
};
use crate::modules::alerts::{
    AlertChannelService, AlertRuleService, AlertService as AlertHistoryService,
    PostgresAlertChannelRepository, PostgresAlertRepository, PostgresAlertRuleRepository,
    RuleEvaluator, WebhookNotifier,
    alert_routes, channel_routes, rule_routes,
};
use crate::modules::metrics::{
    application::MetricsService,
    infrastructure::{TimescaleMetricsRepository, ingest_routes as metrics_ingest_routes, query_routes as metrics_query_routes},
};
use crate::modules::traces::{
    application::TraceService,
    infrastructure::{TimescaleSpanRepository, ingest_routes as traces_ingest_routes, query_routes as traces_query_routes},
};
use crate::modules::otlp::{otlp_logs_routes, otlp_metrics_routes, otlp_traces_routes};
use crate::modules::retention::{start_metrics_cleanup, start_traces_cleanup};

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
    let org_repo = Arc::new(PostgresOrganizationRepository::new(pool.clone()));
    let member_repo = Arc::new(PostgresOrganizationMemberRepository::new(pool.clone()));
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

    // Create auth service (with org repos for personal org creation on register)
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        token_repo,
        password_hasher,
        token_service.clone(),
        id_generator.clone(),
        org_repo.clone(),
        member_repo.clone(),
    ));

    // Create activity repository
    let activity_repo = Arc::new(PostgresOrgActivityRepository::new(pool.clone()));

    // Create invite repository
    let invite_repo = Arc::new(PostgresInviteRepository::new(pool.clone()));

    // Create organization service
    let org_service = Arc::new(OrgService::new(
        org_repo.clone(),
        member_repo.clone(),
        user_repo.clone(),
        token_service.clone(),
        id_generator.clone(),
        activity_repo.clone(),
    ));

    // Create invite service
    let invite_service = Arc::new(InviteService::new(
        org_repo.clone(),
        member_repo.clone(),
        user_repo,
        invite_repo.clone(),
        activity_repo,
        id_generator.clone(),
    ));

    // Create project repositories
    let project_repo = Arc::new(PostgresProjectRepository::new(pool.clone()));
    let api_key_repo = Arc::new(PostgresApiKeyRepository::new(pool.clone()));

    // Create project service
    let project_service = Arc::new(ProjectService::new(
        project_repo.clone(),
        api_key_repo.clone(),
        org_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    // Create logging infrastructure
    let log_repo = Arc::new(TimescaleLogRepository::new(pool.clone()));
    let log_broadcaster = Arc::new(LogBroadcaster::new(1000)); // Buffer up to 1000 messages per channel

    // Create log service
    let log_service = Arc::new(LogService::new(
        log_repo,
        project_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    // Create filter preset repository and service
    let filter_preset_repo = Arc::new(PostgresFilterPresetRepository::new(pool.clone()));
    let filter_preset_service = Arc::new(FilterPresetService::new(
        filter_preset_repo,
        project_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    // Create alert repositories
    let alert_rule_repo = Arc::new(PostgresAlertRuleRepository::new(pool.clone()));
    let alert_channel_repo = Arc::new(PostgresAlertChannelRepository::new(pool.clone()));
    let alert_repo = Arc::new(PostgresAlertRepository::new(pool.clone()));

    // Create alert services
    let alert_channel_service = Arc::new(AlertChannelService::new(
        alert_channel_repo.clone(),
        project_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    let alert_rule_service = Arc::new(AlertRuleService::new(
        alert_rule_repo.clone(),
        alert_channel_repo.clone(),
        project_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    let alert_history_service = Arc::new(AlertHistoryService::new(
        alert_repo.clone(),
        alert_rule_repo.clone(),
        project_repo.clone(),
        member_repo.clone(),
    ));

    // Create webhook notifier
    let webhook_notifier = Arc::new(WebhookNotifier::new());

    // Create metrics infrastructure
    let metrics_repo = Arc::new(TimescaleMetricsRepository::new(pool.clone()));
    let metrics_service = Arc::new(MetricsService::new(
        metrics_repo.clone(),
        project_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    // Create traces infrastructure
    let spans_repo = Arc::new(TimescaleSpanRepository::new((*pool).clone()));
    let trace_service = Arc::new(TraceService::new(
        spans_repo.clone(),
        project_repo.clone(),
        member_repo.clone(),
        id_generator.clone(),
    ));

    // Create and start the rule evaluator background task
    {
        let evaluator = Arc::new(RuleEvaluator::new(
            alert_rule_repo,
            alert_repo,
            alert_channel_repo,
            log_service.log_repo(),
            project_repo.clone(),
            id_generator,
            webhook_notifier,
            60, // Evaluate every 60 seconds
        ));
        tokio::spawn(async move {
            evaluator.start().await;
        });
        tracing::info!("Alert rule evaluator started (runs every 60 seconds)");
    }

    // Spawn log listener background task (listens to pg_notify for real-time streaming)
    {
        let listener_pool = pool.clone();
        let listener_broadcaster = log_broadcaster.clone();
        tokio::spawn(async move {
            if let Err(e) = start_log_listener(listener_pool, listener_broadcaster).await {
                tracing::error!(error = %e, "Log listener failed");
            }
        });
        tracing::info!("Log listener started (listening for PostgreSQL NOTIFY events)");
    }

    // Spawn broadcaster cleanup task
    {
        let cleanup_broadcaster = log_broadcaster.clone();
        tokio::spawn(start_cleanup_task(cleanup_broadcaster));
        tracing::info!("Log broadcaster cleanup task started");
    }

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

    // Spawn metrics retention cleanup task
    {
        let cleanup_metrics_repo = metrics_repo.clone();
        let cleanup_project_repo = project_repo.clone();
        tokio::spawn(start_metrics_cleanup(
            cleanup_metrics_repo,
            cleanup_project_repo,
            60 * 60, // Run every hour
        ));
        tracing::info!("Metrics retention cleanup task started (runs every hour)");
    }

    // Spawn traces retention cleanup task
    {
        let cleanup_spans_repo = spans_repo.clone();
        let cleanup_project_repo = project_repo.clone();
        tokio::spawn(start_traces_cleanup(
            cleanup_spans_repo,
            cleanup_project_repo,
            60 * 60, // Run every hour
        ));
        tracing::info!("Traces retention cleanup task started (runs every hour)");
    }

    // Spawn invite expiration cleanup task
    {
        let cleanup_invite_repo = invite_repo.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60)); // 1 hour
            loop {
                interval.tick().await;
                match cleanup_invite_repo.mark_expired().await {
                    Ok(count) if count > 0 => {
                        tracing::info!(expired_count = count, "Marked expired organization invites");
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to mark expired invites");
                    }
                    _ => {}
                }
            }
        });
        tracing::info!("Invite expiration cleanup task started (runs every hour)");
    }

    // Create router
    let app = Router::new()
        .nest("/api/auth", auth_routes(auth_service, token_service.clone(), rate_limiter))
        .nest("/api", org_routes(org_service, token_service.clone()))
        // Invite routes
        .nest("/api", org_invite_routes(invite_service.clone(), token_service.clone()))
        .nest("/api", user_invite_routes(invite_service, token_service.clone()))
        .nest("/api", project_routes(project_service.clone(), token_service.clone()))
        // Logging routes
        .nest("/api/v1/ingest", ingest_routes(log_service.clone(), project_service.clone()))
        .nest("/api", log_query_routes(log_service.clone(), token_service.clone()))
        .nest("/api", sse_routes(
            log_broadcaster,
            project_repo.clone(),
            member_repo.clone(),
            token_service.clone(),
        ))
        // Filter preset routes
        .nest("/api", filter_preset_routes(filter_preset_service, token_service.clone()))
        // Alert routes
        .nest("/api", channel_routes(alert_channel_service, token_service.clone()))
        .nest("/api", rule_routes(alert_rule_service, token_service.clone()))
        .nest("/api", alert_routes(alert_history_service, token_service.clone()))
        // Metrics routes
        .nest("/api/v1/ingest", metrics_ingest_routes(metrics_service.clone(), project_service.clone()))
        .nest("/api/projects/{project_id}/observability/metrics", metrics_query_routes(metrics_service.clone(), token_service.clone()))
        // Traces routes
        .nest("/api/v1/ingest", traces_ingest_routes(trace_service.clone(), project_service.clone()))
        .nest("/api/projects/{project_id}/observability/traces", traces_query_routes(trace_service.clone(), token_service.clone()))
        // OTLP routes (under /v1 for compatibility)
        .nest("/v1", otlp_logs_routes(log_service, project_service.clone()))
        .nest("/v1", otlp_metrics_routes(metrics_service, project_service.clone()))
        .nest("/v1", otlp_traces_routes(trace_service, project_service))
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
