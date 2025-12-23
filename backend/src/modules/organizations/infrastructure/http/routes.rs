use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::handlers;
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::domain::UserRepository;
use crate::modules::auth::infrastructure::http::middleware::auth_middleware;
use crate::modules::organizations::application::services::OrgService;
use crate::modules::organizations::domain::{OrganizationMemberRepository, OrganizationRepository};

/// Create organization routes (all protected)
pub fn org_routes<OR, MR, UR, TS, ID>(
    org_service: Arc<OrgService<OR, MR, UR, TS, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    TS: TokenService + 'static,
    ID: IdGenerator + 'static,
{
    Router::new()
        // Organization CRUD
        .route("/orgs", post(handlers::create_org::<OR, MR, UR, TS, ID>))
        .route("/orgs", get(handlers::list_orgs::<OR, MR, UR, TS, ID>))
        .route("/orgs/{id}", get(handlers::get_org::<OR, MR, UR, TS, ID>))
        .route("/orgs/{id}", patch(handlers::update_org::<OR, MR, UR, TS, ID>))
        .route("/orgs/{id}", delete(handlers::delete_org::<OR, MR, UR, TS, ID>))
        // Members
        .route("/orgs/{id}/members", get(handlers::list_members::<OR, MR, UR, TS, ID>))
        .route("/orgs/{id}/members", post(handlers::add_member::<OR, MR, UR, TS, ID>))
        .route(
            "/orgs/{id}/members/{uid}",
            patch(handlers::update_member_role::<OR, MR, UR, TS, ID>),
        )
        .route(
            "/orgs/{id}/members/{uid}",
            delete(handlers::remove_member::<OR, MR, UR, TS, ID>),
        )
        // Leave, transfer, switch
        .route("/orgs/{id}/leave", post(handlers::leave_org::<OR, MR, UR, TS, ID>))
        .route(
            "/orgs/{id}/transfer",
            post(handlers::transfer_ownership::<OR, MR, UR, TS, ID>),
        )
        .route("/orgs/{id}/switch", post(handlers::switch_org::<OR, MR, UR, TS, ID>))
        // All routes require authentication
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(org_service)
}
