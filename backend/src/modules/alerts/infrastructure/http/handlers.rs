use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::modules::alerts::application::dto::*;
use crate::modules::alerts::application::services::{
    AlertChannelService, AlertRuleService, AlertService,
};
use crate::modules::alerts::domain::{
    AlertChannelRepository, AlertDomainError, AlertRepository, AlertRuleRepository,
};
use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::organizations::domain::OrganizationMemberRepository;
use crate::modules::projects::domain::ProjectRepository;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// Query params for alerts list
#[derive(Debug, Deserialize)]
pub struct AlertQueryParams {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

fn to_error_response(e: AlertDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        AlertDomainError::InvalidRuleName(msg)
        | AlertDomainError::InvalidRuleType(msg)
        | AlertDomainError::InvalidThresholdOperator(msg)
        | AlertDomainError::InvalidChannelName(msg)
        | AlertDomainError::InvalidChannelConfig(msg)
        | AlertDomainError::InvalidChannelType(msg)
        | AlertDomainError::InvalidWebhookUrl(msg)
        | AlertDomainError::ValidationError(msg) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        AlertDomainError::RuleNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Alert rule not found".to_string(),
                code: "RULE_NOT_FOUND".to_string(),
            }),
        ),
        AlertDomainError::ChannelNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Alert channel not found".to_string(),
                code: "CHANNEL_NOT_FOUND".to_string(),
            }),
        ),
        AlertDomainError::AlertNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Alert not found".to_string(),
                code: "ALERT_NOT_FOUND".to_string(),
            }),
        ),
        AlertDomainError::ProjectNotFound | AlertDomainError::ProjectDeleted => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Project not found".to_string(),
                code: "PROJECT_NOT_FOUND".to_string(),
            }),
        ),
        AlertDomainError::RuleNameExists(name) => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Alert rule with name '{}' already exists", name),
                code: "RULE_NAME_EXISTS".to_string(),
            }),
        ),
        AlertDomainError::ChannelNameExists(name) => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Alert channel with name '{}' already exists", name),
                code: "CHANNEL_NAME_EXISTS".to_string(),
            }),
        ),
        AlertDomainError::AlertAlreadyResolved => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Alert is already resolved".to_string(),
                code: "ALERT_ALREADY_RESOLVED".to_string(),
            }),
        ),
        AlertDomainError::NotAuthorized | AlertDomainError::NotOrgMember => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Access denied".to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        ),
        AlertDomainError::WebhookFailed(ref msg) => {
            tracing::warn!(error = %msg, "Webhook failed");
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "Webhook notification failed".to_string(),
                    code: "WEBHOOK_FAILED".to_string(),
                }),
            )
        }
        AlertDomainError::InternalError(ref msg) => {
            tracing::error!(error = %msg, "Internal error occurred");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "An internal error occurred".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
        }
    }
}

// ============================================================================
// Alert Channel Handlers
// ============================================================================

pub async fn create_channel<CR, PR, MR, ID>(
    State(service): State<Arc<AlertChannelService<CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateAlertChannelRequest>,
) -> Result<(StatusCode, Json<AlertChannelResponse>), (StatusCode, Json<ErrorResponse>)>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let response = service
        .create_channel(&project_id, request, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_channels<CR, PR, MR, ID>(
    State(service): State<Arc<AlertChannelService<CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<AlertChannelResponse>>, (StatusCode, Json<ErrorResponse>)>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let channels = service
        .list_channels(&project_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(channels))
}

pub async fn get_channel<CR, PR, MR, ID>(
    State(service): State<Arc<AlertChannelService<CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, channel_id)): Path<(String, String)>,
) -> Result<Json<AlertChannelResponse>, (StatusCode, Json<ErrorResponse>)>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let channel = service
        .get_channel(&project_id, &channel_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(channel))
}

pub async fn update_channel<CR, PR, MR, ID>(
    State(service): State<Arc<AlertChannelService<CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, channel_id)): Path<(String, String)>,
    Json(request): Json<UpdateAlertChannelRequest>,
) -> Result<Json<AlertChannelResponse>, (StatusCode, Json<ErrorResponse>)>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let channel = service
        .update_channel(&project_id, &channel_id, request, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(channel))
}

pub async fn delete_channel<CR, PR, MR, ID>(
    State(service): State<Arc<AlertChannelService<CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, channel_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .delete_channel(&project_id, &channel_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Alert Rule Handlers
// ============================================================================

pub async fn create_rule<RR, CR, PR, MR, ID>(
    State(service): State<Arc<AlertRuleService<RR, CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateAlertRuleRequest>,
) -> Result<(StatusCode, Json<AlertRuleResponse>), (StatusCode, Json<ErrorResponse>)>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let response = service
        .create_rule(&project_id, request, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_rules<RR, CR, PR, MR, ID>(
    State(service): State<Arc<AlertRuleService<RR, CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<AlertRuleResponse>>, (StatusCode, Json<ErrorResponse>)>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let rules = service
        .list_rules(&project_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(rules))
}

pub async fn get_rule<RR, CR, PR, MR, ID>(
    State(service): State<Arc<AlertRuleService<RR, CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, rule_id)): Path<(String, String)>,
) -> Result<Json<AlertRuleResponse>, (StatusCode, Json<ErrorResponse>)>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let rule = service
        .get_rule(&project_id, &rule_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(rule))
}

pub async fn update_rule<RR, CR, PR, MR, ID>(
    State(service): State<Arc<AlertRuleService<RR, CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, rule_id)): Path<(String, String)>,
    Json(request): Json<UpdateAlertRuleRequest>,
) -> Result<Json<AlertRuleResponse>, (StatusCode, Json<ErrorResponse>)>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    let rule = service
        .update_rule(&project_id, &rule_id, request, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(rule))
}

pub async fn delete_rule<RR, CR, PR, MR, ID>(
    State(service): State<Arc<AlertRuleService<RR, CR, PR, MR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, rule_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    RR: AlertRuleRepository,
    CR: AlertChannelRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
    ID: IdGenerator,
{
    service
        .delete_rule(&project_id, &rule_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Alert Handlers
// ============================================================================

pub async fn list_alerts<AR, RR, PR, MR>(
    State(service): State<Arc<AlertService<AR, RR, PR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(project_id): Path<String>,
    Query(params): Query<AlertQueryParams>,
) -> Result<Json<AlertListResponse>, (StatusCode, Json<ErrorResponse>)>
where
    AR: AlertRepository,
    RR: AlertRuleRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
{
    let response = service
        .list_alerts(&project_id, params.limit, params.offset, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(response))
}

pub async fn get_alert<AR, RR, PR, MR>(
    State(service): State<Arc<AlertService<AR, RR, PR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, alert_id)): Path<(String, String)>,
) -> Result<Json<AlertResponse>, (StatusCode, Json<ErrorResponse>)>
where
    AR: AlertRepository,
    RR: AlertRuleRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
{
    let alert = service
        .get_alert(&project_id, &alert_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(alert))
}

pub async fn resolve_alert<AR, RR, PR, MR>(
    State(service): State<Arc<AlertService<AR, RR, PR, MR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((project_id, alert_id)): Path<(String, String)>,
) -> Result<Json<AlertResponse>, (StatusCode, Json<ErrorResponse>)>
where
    AR: AlertRepository,
    RR: AlertRuleRepository,
    PR: ProjectRepository,
    MR: OrganizationMemberRepository,
{
    let alert = service
        .resolve_alert(&project_id, &alert_id, &claims.user_id)
        .await
        .map_err(to_error_response)?;

    Ok(Json(alert))
}
