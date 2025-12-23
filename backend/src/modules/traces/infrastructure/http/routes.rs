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
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, ProjectRepository};
use crate::modules::traces::application::TraceService;
use crate::modules::traces::domain::SpansRepository;

/// Routes for trace ingestion (API Key auth)
pub fn ingest_routes<SR, PR, OMR, ID, PPR, AR, OR>(
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
        .route("/traces", post(handlers::ingest_spans::<SR, PR, OMR, ID>))
        .layer(middleware::from_fn_with_state(
            project_service,
            api_key_middleware::<PPR, AR, OR, OMR, ID>,
        ))
        .with_state(service)
}

/// Routes for trace queries (JWT auth)
pub fn query_routes<SR, PR, OMR, ID, TS>(
    service: Arc<TraceService<SR, PR, OMR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    SR: SpansRepository + 'static,
    PR: ProjectRepository + 'static,
    OMR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route("/", get(handlers::search_traces::<SR, PR, OMR, ID>))
        .route("/{trace_id}", get(handlers::get_trace::<SR, PR, OMR, ID>))
        .route("/services", get(handlers::list_services::<SR, PR, OMR, ID>))
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(service)
}
