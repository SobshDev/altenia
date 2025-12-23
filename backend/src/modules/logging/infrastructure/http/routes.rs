use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use super::filter_preset_handlers;
use super::handlers;
use super::middleware::api_key_middleware;
use super::sse;
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::infrastructure::http::middleware::auth_middleware;
use crate::modules::logging::application::services::{FilterPresetService, LogService};
use crate::modules::logging::domain::{FilterPresetRepository, LogRepository};
use crate::modules::logging::infrastructure::broadcast::LogBroadcaster;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::services::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, ProjectRepository};

/// Create ingestion routes (API key auth)
pub fn ingest_routes<LR, PR, MR, ID, PPR, AR, OR>(
    log_service: Arc<LogService<LR, PR, MR, ID>>,
    project_service: Arc<ProjectService<PPR, AR, OR, MR, ID>>,
) -> Router
where
    LR: LogRepository + 'static,
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    PPR: ProjectRepository + 'static,
    AR: ApiKeyRepository + 'static,
    OR: OrganizationRepository + 'static,
{
    Router::new()
        .route(
            "/ingest/logs",
            post(handlers::ingest_logs::<LR, PR, MR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            project_service,
            api_key_middleware::<PPR, AR, OR, MR, ID>,
        ))
        .with_state(log_service)
}

/// Create log query routes (JWT auth)
pub fn log_query_routes<LR, PR, MR, ID, TS>(
    log_service: Arc<LogService<LR, PR, MR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    LR: LogRepository + 'static,
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/projects/{id}/logs",
            get(handlers::query_logs::<LR, PR, MR, ID>),
        )
        .route(
            "/projects/{id}/logs/stats",
            get(handlers::get_log_stats::<LR, PR, MR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(log_service)
}

/// Create SSE streaming routes (JWT auth)
pub fn sse_routes<PR, MR, TS>(
    broadcaster: Arc<LogBroadcaster>,
    project_repo: Arc<PR>,
    member_repo: Arc<MR>,
    token_service: Arc<TS>,
) -> Router
where
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/projects/{id}/logs/stream",
            get(sse::stream_logs::<PR, MR>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state((broadcaster, project_repo, member_repo))
}

/// Create filter preset routes (JWT auth)
pub fn filter_preset_routes<FPR, PR, MR, ID, TS>(
    filter_preset_service: Arc<FilterPresetService<FPR, PR, MR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    FPR: FilterPresetRepository + 'static,
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/projects/{id}/filter-presets",
            post(filter_preset_handlers::create_preset::<FPR, PR, MR, ID>)
                .get(filter_preset_handlers::list_presets::<FPR, PR, MR, ID>),
        )
        .route(
            "/projects/{id}/filter-presets/default",
            get(filter_preset_handlers::get_default_preset::<FPR, PR, MR, ID>),
        )
        .route(
            "/projects/{project_id}/filter-presets/{preset_id}",
            get(filter_preset_handlers::get_preset::<FPR, PR, MR, ID>)
                .put(filter_preset_handlers::update_preset::<FPR, PR, MR, ID>)
                .delete(filter_preset_handlers::delete_preset::<FPR, PR, MR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(filter_preset_service)
}
