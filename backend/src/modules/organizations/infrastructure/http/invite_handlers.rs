use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::UserRepository;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::organizations::application::dto::*;
use crate::modules::organizations::application::services::InviteService;
use crate::modules::organizations::domain::{
    OrgActivityRepository, OrgDomainError, OrganizationInviteRepository,
    OrganizationMemberRepository, OrganizationRepository,
};

use super::handlers::ErrorResponse;

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SendInviteRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct InviteResponseDto {
    pub id: String,
    pub organization_id: String,
    pub organization_name: String,
    pub inviter_email: String,
    pub invitee_email: String,
    pub role: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InviteCountResponseDto {
    pub count: i64,
}

impl From<InviteResponse> for InviteResponseDto {
    fn from(r: InviteResponse) -> Self {
        Self {
            id: r.id,
            organization_id: r.organization_id,
            organization_name: r.organization_name,
            inviter_email: r.inviter_email,
            invitee_email: r.invitee_email,
            role: r.role,
            status: r.status,
            expires_at: r.expires_at,
            created_at: r.created_at,
        }
    }
}

impl From<InviteCountResponse> for InviteCountResponseDto {
    fn from(r: InviteCountResponse) -> Self {
        Self { count: r.count }
    }
}

fn to_error_response(err: OrgDomainError) -> (StatusCode, Json<ErrorResponse>) {
    let (status, code) = match &err {
        OrgDomainError::InvalidOrgName(_)
        | OrgDomainError::InvalidOrgSlug(_)
        | OrgDomainError::InvalidRole(_)
        | OrgDomainError::InvalidActivityType(_)
        | OrgDomainError::InvalidInviteStatus(_)
        | OrgDomainError::CannotInviteSelf => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),

        OrgDomainError::OrgNotFound | OrgDomainError::InviteNotFound => {
            (StatusCode::NOT_FOUND, "NOT_FOUND")
        }

        OrgDomainError::InviteAlreadyExists | OrgDomainError::AlreadyMember => {
            (StatusCode::CONFLICT, "CONFLICT")
        }

        OrgDomainError::InsufficientPermissions
        | OrgDomainError::NotOrgMember
        | OrgDomainError::UserDoesNotAllowInvites => (StatusCode::FORBIDDEN, "FORBIDDEN"),

        OrgDomainError::InviteExpired => (StatusCode::GONE, "INVITE_EXPIRED"),
        OrgDomainError::InviteAlreadyProcessed => (StatusCode::CONFLICT, "INVITE_ALREADY_PROCESSED"),

        OrgDomainError::UserNotFound => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),

        _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
    };

    (
        status,
        Json(ErrorResponse {
            error: err.to_string(),
            code: code.to_string(),
        }),
    )
}

// ============================================================================
// Handlers
// ============================================================================

/// Send an invite (POST /api/orgs/{id}/invites)
pub async fn send_invite<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
    Json(req): Json<SendInviteRequest>,
) -> Result<(StatusCode, Json<InviteResponseDto>), (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    let cmd = SendInviteCommand {
        org_id,
        invitee_email: req.email,
        role: req.role,
        inviter_user_id: claims.user_id,
    };

    service
        .send_invite(cmd)
        .await
        .map(|r| (StatusCode::CREATED, Json(r.into())))
        .map_err(to_error_response)
}

/// List org's pending invites (GET /api/orgs/{id}/invites)
pub async fn list_org_invites<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<InviteResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    service
        .list_org_invites(&org_id, &claims.user_id)
        .await
        .map(|invites| Json(invites.into_iter().map(Into::into).collect()))
        .map_err(to_error_response)
}

/// Cancel an invite (DELETE /api/orgs/{id}/invites/{invite_id})
pub async fn cancel_invite<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((org_id, invite_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    let cmd = CancelInviteCommand {
        org_id,
        invite_id,
        requesting_user_id: claims.user_id,
    };

    service
        .cancel_invite(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// List user's pending invites (GET /api/invites)
pub async fn list_user_invites<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<InviteResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    service
        .list_user_invites(&claims.user_id)
        .await
        .map(|invites| Json(invites.into_iter().map(Into::into).collect()))
        .map_err(to_error_response)
}

/// Get invite count for badge (GET /api/invites/count)
pub async fn count_user_invites<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<InviteCountResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    service
        .count_user_invites(&claims.user_id)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// Accept an invite (POST /api/invites/{id}/accept)
pub async fn accept_invite<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(invite_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    let cmd = AcceptInviteCommand {
        invite_id,
        user_id: claims.user_id,
    };

    service
        .accept_invite(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// Decline an invite (POST /api/invites/{id}/decline)
pub async fn decline_invite<OR, MR, UR, IR, AR, ID>(
    State(service): State<Arc<InviteService<OR, MR, UR, IR, AR, ID>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(invite_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository + 'static,
    MR: OrganizationMemberRepository + 'static,
    UR: UserRepository + 'static,
    IR: OrganizationInviteRepository + 'static,
    AR: OrgActivityRepository + 'static,
    ID: IdGenerator + 'static,
{
    let cmd = DeclineInviteCommand {
        invite_id,
        user_id: claims.user_id,
    };

    service
        .decline_invite(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}
