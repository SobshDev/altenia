use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use super::handlers;
use crate::modules::alerts::application::services::{
    AlertChannelService, AlertRuleService, AlertService,
};
use crate::modules::alerts::domain::{AlertChannelRepository, AlertRepository, AlertRuleRepository};
use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::infrastructure::http::middleware::auth_middleware;
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::projects::domain::ProjectRepository;

/// Create alert channel routes (JWT auth)
pub fn channel_routes<CR, PR, MR, ID, TS>(
    channel_service: Arc<AlertChannelService<CR, PR, MR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    CR: AlertChannelRepository + 'static,
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/projects/{id}/alert-channels",
            post(handlers::create_channel::<CR, PR, MR, ID>)
                .get(handlers::list_channels::<CR, PR, MR, ID>),
        )
        .route(
            "/projects/{project_id}/alert-channels/{channel_id}",
            get(handlers::get_channel::<CR, PR, MR, ID>)
                .put(handlers::update_channel::<CR, PR, MR, ID>)
                .delete(handlers::delete_channel::<CR, PR, MR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(channel_service)
}

/// Create alert rule routes (JWT auth)
pub fn rule_routes<RR, CR, PR, MR, ID, TS>(
    rule_service: Arc<AlertRuleService<RR, CR, PR, MR, ID>>,
    token_service: Arc<TS>,
) -> Router
where
    RR: AlertRuleRepository + 'static,
    CR: AlertChannelRepository + 'static,
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    ID: IdGenerator + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/projects/{id}/alert-rules",
            post(handlers::create_rule::<RR, CR, PR, MR, ID>)
                .get(handlers::list_rules::<RR, CR, PR, MR, ID>),
        )
        .route(
            "/projects/{project_id}/alert-rules/{rule_id}",
            get(handlers::get_rule::<RR, CR, PR, MR, ID>)
                .put(handlers::update_rule::<RR, CR, PR, MR, ID>)
                .delete(handlers::delete_rule::<RR, CR, PR, MR, ID>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(rule_service)
}

/// Create alert (triggered) routes (JWT auth)
pub fn alert_routes<AR, RR, PR, MR, TS>(
    alert_service: Arc<AlertService<AR, RR, PR, MR>>,
    token_service: Arc<TS>,
) -> Router
where
    AR: AlertRepository + 'static,
    RR: AlertRuleRepository + 'static,
    PR: ProjectRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    TS: TokenService + 'static,
{
    Router::new()
        .route(
            "/projects/{id}/alerts",
            get(handlers::list_alerts::<AR, RR, PR, MR>),
        )
        .route(
            "/projects/{project_id}/alerts/{alert_id}",
            get(handlers::get_alert::<AR, RR, PR, MR>),
        )
        .route(
            "/projects/{project_id}/alerts/{alert_id}/resolve",
            post(handlers::resolve_alert::<AR, RR, PR, MR>),
        )
        .layer(middleware::from_fn_with_state(
            token_service,
            auth_middleware::<TS>,
        ))
        .with_state(alert_service)
}
