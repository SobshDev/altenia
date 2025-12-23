//! OTLP HTTP routes

use axum::{middleware, routing::post, Router};
use std::sync::Arc;

use super::handlers;
use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::logging::application::LogService;
use crate::modules::logging::domain::LogRepository;
use crate::modules::logging::infrastructure::http::middleware::api_key_middleware;
use crate::modules::metrics::application::MetricsService;
use crate::modules::metrics::domain::MetricsRepository;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, ProjectRepository};
use crate::modules::traces::application::TraceService;
use crate::modules::traces::domain::SpansRepository;

/// OTLP routes for logs ingestion (requires API key middleware)
pub fn otlp_logs_routes<LR, PR, OMR, ID, PPR, AR, OR>(
    service: Arc<LogService<LR, PR, OMR, ID>>,
    project_service: Arc<ProjectService<PPR, AR, OR, OMR, ID>>,
) -> Router
where
    LR: LogRepository + 'static,
    PR: ProjectRepository + 'static,
    OMR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    PPR: ProjectRepository + 'static,
    AR: ApiKeyRepository + 'static,
    OR: OrganizationRepository + 'static,
{
    Router::new()
        .route("/logs", post(handlers::ingest_otlp_logs::<LR, PR, OMR, ID>))
        .layer(middleware::from_fn_with_state(
            project_service,
            api_key_middleware::<PPR, AR, OR, OMR, ID>,
        ))
        .with_state(service)
}

/// OTLP routes for metrics ingestion (requires API key middleware)
pub fn otlp_metrics_routes<MR, PR, OMR, ID, PPR, AR, OR>(
    service: Arc<MetricsService<MR, PR, OMR, ID>>,
    project_service: Arc<ProjectService<PPR, AR, OR, OMR, ID>>,
) -> Router
where
    MR: MetricsRepository + 'static,
    PR: ProjectRepository + 'static,
    OMR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    PPR: ProjectRepository + 'static,
    AR: ApiKeyRepository + 'static,
    OR: OrganizationRepository + 'static,
{
    Router::new()
        .route(
            "/metrics",
            post(handlers::ingest_otlp_metrics::<MR, PR, OMR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            project_service,
            api_key_middleware::<PPR, AR, OR, OMR, ID>,
        ))
        .with_state(service)
}

/// OTLP routes for traces ingestion (requires API key middleware)
pub fn otlp_traces_routes<SR, PR, OMR, ID, PPR, AR, OR>(
    service: Arc<TraceService<SR, PR, OMR, ID>>,
    project_service: Arc<ProjectService<PPR, AR, OR, OMR, ID>>,
) -> Router
where
    SR: SpansRepository + 'static,
    PR: ProjectRepository + 'static,
    OMR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    PPR: ProjectRepository + 'static,
    AR: ApiKeyRepository + 'static,
    OR: OrganizationRepository + 'static,
{
    Router::new()
        .route(
            "/traces",
            post(handlers::ingest_otlp_traces::<SR, PR, OMR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            project_service,
            api_key_middleware::<PPR, AR, OR, OMR, ID>,
        ))
        .with_state(service)
}
