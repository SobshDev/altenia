use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use super::invite_handlers;
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::domain::UserRepository;
use crate::modules::auth::infrastructure::http::middleware::auth_middleware;
use crate::modules::organizations::application::services::InviteService;
use crate::modules::organizations::domain::{
    OrgActivityRepository, OrganizationInviteRepository, OrganizationMemberRepository,
    OrganizationRepository,
};

/// Create invite routes for organization invites (POST/GET/DELETE /api/orgs/{id}/invites)
pub fn org_invite_routes<OR, MR, UR, IR, AR, ID, TS>(
    invite_service: Arc<InviteService<OR, MR, UR, IR, AR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/orgs/{id}/invites",
            post(invite_handlers::send_invite::<OR, MR, UR, IR, AR, ID>),
        )
        .route(
            "/orgs/{id}/invites",
            get(invite_handlers::list_org_invites::<OR, MR, UR, IR, AR, ID>),
        )
        .route(
            "/orgs/{id}/invites/{invite_id}",
            delete(invite_handlers::cancel_invite::<OR, MR, UR, IR, AR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(invite_service)
}

/// Create user invite routes (GET /api/invites, POST /api/invites/{id}/accept, etc.)
pub fn user_invite_routes<OR, MR, UR, IR, AR, ID, TS>(
    invite_service: Arc<InviteService<OR, MR, UR, IR, AR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/invites",
            get(invite_handlers::list_user_invites::<OR, MR, UR, IR, AR, ID>),
        )
        .route(
            "/invites/count",
            get(invite_handlers::count_user_invites::<OR, MR, UR, IR, AR, ID>),
        )
        .route(
            "/invites/{id}/accept",
            post(invite_handlers::accept_invite::<OR, MR, UR, IR, AR, ID>),
        )
        .route(
            "/invites/{id}/decline",
            post(invite_handlers::decline_invite::<OR, MR, UR, IR, AR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(invite_service)
}
