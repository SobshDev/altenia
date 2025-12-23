use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::handlers;
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::infrastructure::http::middleware::auth_middleware;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};
use crate::modules::projects::application::services::ProjectService;
use crate::modules::projects::domain::{ApiKeyRepository, ProjectRepository};

/// Create project routes (all protected)
pub fn project_routes<PR, AR, OR, MR, TS, ID>(
    project_service: Arc<ProjectService<PR, AR, OR, MR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    PR: ProjectRepository + 'static,
    AR: ApiKeyRepository + 'static,
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    TS: TokenService + 'static,
    ID: IdGenerator + 'static,
{
    Router::new()
        // Project CRUD via org
        .route(
            "/orgs/{org_id}/projects",
            post(handlers::create_project::<PR, AR, OR, MR, ID>),
        )
        .route(
            "/orgs/{org_id}/projects",
            get(handlers::list_projects::<PR, AR, OR, MR, ID>),
        )
        // Project CRUD via project ID
        .route(
            "/projects/{id}",
            get(handlers::get_project::<PR, AR, OR, MR, ID>),
        )
        .route(
            "/projects/{id}",
            patch(handlers::update_project::<PR, AR, OR, MR, ID>),
        )
        .route(
            "/projects/{id}",
            delete(handlers::delete_project::<PR, AR, OR, MR, ID>),
        )
        // API Keys
        .route(
            "/projects/{id}/api-keys",
            post(handlers::create_api_key::<PR, AR, OR, MR, ID>),
        )
        .route(
            "/projects/{id}/api-keys",
            get(handlers::list_api_keys::<PR, AR, OR, MR, ID>),
        )
        .route(
            "/projects/{project_id}/api-keys/{key_id}",
            delete(handlers::revoke_api_key::<PR, AR, OR, MR, ID>),
        )
        // All routes require authentication
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(project_service)
}
