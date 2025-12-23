use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::handlers;
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::infrastructure::http::middleware::auth_middleware;
use crate::modules::logging::infrastructure::http::middleware::api_key_middleware;
use crate::modules::metrics::application::MetricsService;
use crate::modules::metrics::domain::MetricsRepository;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, ProjectRepository};

/// Routes for metrics ingestion (API Key auth)
pub fn ingest_routes<MR, PR, OMR, ID, PPR, AR, OR>(
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
        .route("/metrics", post(handlers::ingest_metrics::<MR, PR, OMR, ID>))
        .layer(middleware::from_fn_with_state(
            project_service,
            api_key_middleware::<PPR, AR, OR, OMR, ID>,
        ))
        .with_state(service)
}

/// Routes for metrics queries (JWT auth)
pub fn query_routes<MR, PR, OMR, ID, TS>(
    service: Arc<MetricsService<MR, PR, OMR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    MR: MetricsRepository + 'static,
    PR: ProjectRepository + 'static,
    OMR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route("/", get(handlers::query_metrics::<MR, PR, OMR, ID>))
        .route("/names", get(handlers::list_metric_names::<MR, PR, OMR, ID>))
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(service)
}
