use axum::{
    extract::{Path, Query, State},
    http::{header::USER_AGENT, HeaderMap, StatusCode},
    Extension, Json,
};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::modules::auth::application::ports::{IdGenerator, TokenService};
use crate::modules::auth::domain::UserRepository;
use crate::modules::auth::infrastructure::http::extractors::AuthClaims;
use crate::modules::organizations::application::dto::*;
use crate::modules::organizations::application::services::OrgService;
use crate::modules::organizations::domain::{
    OrgActivityRepository, OrgDomainError, OrganizationMemberRepository, OrganizationRepository,
};

// ============================================================================
// Request/Response DTOs for HTTP layer
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferOwnershipRequest {
    pub new_owner_user_id: String,
}

#[derive(Debug, Serialize)]
pub struct OrgResponseDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MemberResponseDto {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SwitchOrgResponseDto {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub organization: OrgResponseDto,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl From<OrgResponse> for OrgResponseDto {
    fn from(r: OrgResponse) -> Self {
        Self {
            id: r.id,
            name: r.name,
            slug: r.slug,
            is_personal: r.is_personal,
            role: r.role,
            created_at: r.created_at,
        }
    }
}

impl From<MemberResponse> for MemberResponseDto {
    fn from(r: MemberResponse) -> Self {
        Self {
            id: r.id,
            user_id: r.user_id,
            email: r.email,
            display_name: r.display_name,
            role: r.role,
            joined_at: r.joined_at,
        }
    }
}

impl From<SwitchOrgResponse> for SwitchOrgResponseDto {
    fn from(r: SwitchOrgResponse) -> Self {
        Self {
            access_token: r.access_token,
            refresh_token: r.refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: r.expires_in,
            organization: r.organization.into(),
        }
    }
}

// ============================================================================
// Error handling
// ============================================================================

fn to_error_response(e: OrgDomainError) -> (StatusCode, Json<ErrorResponse>) {
    match e {
        OrgDomainError::InvalidOrgName(_)
        | OrgDomainError::InvalidOrgSlug(_)
        | OrgDomainError::InvalidRole(_)
        | OrgDomainError::InvalidActivityType(_)
        | OrgDomainError::InvalidInviteStatus(_)
        | OrgDomainError::CannotInviteSelf => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "VALIDATION_ERROR".to_string(),
            }),
        ),
        OrgDomainError::OrgNotFound | OrgDomainError::InviteNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "NOT_FOUND".to_string(),
            }),
        ),
        OrgDomainError::UserNotFound => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "User not found".to_string(),
                code: "USER_NOT_FOUND".to_string(),
            }),
        ),
        OrgDomainError::SlugTaken
        | OrgDomainError::OrgAlreadyExists
        | OrgDomainError::InviteAlreadyExists
        | OrgDomainError::InviteAlreadyProcessed => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "CONFLICT".to_string(),
            }),
        ),
        OrgDomainError::AlreadyMember => (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "User is already a member of this organization".to_string(),
                code: "ALREADY_MEMBER".to_string(),
            }),
        ),
        OrgDomainError::NotOrgMember
        | OrgDomainError::InsufficientPermissions
        | OrgDomainError::UserDoesNotAllowInvites => (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        ),
        OrgDomainError::CannotDeletePersonalOrg
        | OrgDomainError::CannotRemoveLastOwner
        | OrgDomainError::CannotLeaveAsLastOwner
        | OrgDomainError::CannotDemoteLastOwner => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "UNPROCESSABLE".to_string(),
            }),
        ),
        OrgDomainError::OrgAlreadyDeleted | OrgDomainError::InviteExpired => (
            StatusCode::GONE,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "GONE".to_string(),
            }),
        ),
        OrgDomainError::InternalError(ref msg) => {
            tracing::error!(error = %msg, "Internal error occurred");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "An internal error occurred. Please try again later.".to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                }),
            )
        }
    }
}

/// Generate device fingerprint from User-Agent and X-Forwarded-For headers
fn generate_device_fingerprint(headers: &HeaderMap) -> String {
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let ip_str = headers
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim())
        .unwrap_or("unknown");

    let ip_subnet = if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
            }
            std::net::IpAddr::V6(ipv6) => {
                let segments = ipv6.segments();
                format!("{:x}:{:x}:{:x}::/48", segments[0], segments[1], segments[2])
            }
        }
    } else {
        ip_str.to_string()
    };

    let mut hasher = Sha256::new();
    hasher.update(user_agent.as_bytes());
    hasher.update(ip_subnet.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/orgs
pub async fn create_org<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<OrgResponseDto>), (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = CreateOrgCommand {
        name: req.name,
        user_id: claims.user_id,
    };

    org_service
        .create_org(cmd)
        .await
        .map(|r| (StatusCode::CREATED, Json(r.into())))
        .map_err(to_error_response)
}

/// GET /api/orgs
pub async fn list_orgs<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<Json<Vec<OrgResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    org_service
        .list_user_orgs(&claims.user_id)
        .await
        .map(|orgs| Json(orgs.into_iter().map(|o| o.into()).collect()))
        .map_err(to_error_response)
}

/// GET /api/orgs/:id
pub async fn get_org<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    org_service
        .get_org(&org_id, &claims.user_id)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// PATCH /api/orgs/:id
pub async fn update_org<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<Json<OrgResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = UpdateOrgCommand {
        org_id,
        name: req.name,
        requesting_user_id: claims.user_id,
    };

    org_service
        .update_org(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// DELETE /api/orgs/:id
pub async fn delete_org<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = DeleteOrgCommand {
        org_id,
        requesting_user_id: claims.user_id,
    };

    org_service
        .delete_org(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// GET /api/orgs/:id/members
pub async fn list_members<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<MemberResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    org_service
        .list_members(&org_id, &claims.user_id)
        .await
        .map(|members| Json(members.into_iter().map(|m| m.into()).collect()))
        .map_err(to_error_response)
}

/// POST /api/orgs/:id/members
pub async fn add_member<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<MemberResponseDto>), (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = AddMemberCommand {
        org_id,
        email: req.email,
        role: req.role,
        requesting_user_id: claims.user_id,
    };

    org_service
        .add_member(cmd)
        .await
        .map(|r| (StatusCode::CREATED, Json(r.into())))
        .map_err(to_error_response)
}

/// PATCH /api/orgs/:id/members/:uid
pub async fn update_member_role<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((org_id, target_user_id)): Path<(String, String)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> Result<Json<MemberResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = UpdateMemberRoleCommand {
        org_id,
        target_user_id,
        new_role: req.role,
        requesting_user_id: claims.user_id,
    };

    org_service
        .update_member_role(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// DELETE /api/orgs/:id/members/:uid
pub async fn remove_member<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path((org_id, target_user_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = RemoveMemberCommand {
        org_id,
        target_user_id,
        requesting_user_id: claims.user_id,
    };

    org_service
        .remove_member(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// POST /api/orgs/:id/leave
pub async fn leave_org<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = LeaveOrgCommand {
        org_id,
        user_id: claims.user_id,
    };

    org_service
        .leave_org(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(to_error_response)
}

/// POST /api/orgs/:id/transfer
pub async fn transfer_ownership<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<Json<SwitchOrgResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let cmd = TransferOwnershipCommand {
        org_id,
        new_owner_user_id: req.new_owner_user_id,
        requesting_user_id: claims.user_id,
    };

    org_service
        .transfer_ownership(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

/// POST /api/orgs/:id/switch
pub async fn switch_org<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    headers: HeaderMap,
    Path(org_id): Path<String>,
) -> Result<Json<SwitchOrgResponseDto>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    let device_fingerprint = generate_device_fingerprint(&headers);

    let cmd = SwitchOrgCommand {
        org_id,
        user_id: claims.user_id,
        device_fingerprint,
    };

    org_service
        .switch_org(cmd)
        .await
        .map(|r| Json(r.into()))
        .map_err(to_error_response)
}

// ============================================================================
// Activities
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListActivitiesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct ActivityResponseDto {
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor_email: String,
    pub target_email: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
}

impl From<ActivityResponse> for ActivityResponseDto {
    fn from(r: ActivityResponse) -> Self {
        Self {
            id: r.id,
            activity_type: r.activity_type,
            actor_email: r.actor_email,
            target_email: r.target_email,
            metadata: r.metadata,
            created_at: r.created_at,
        }
    }
}

/// GET /api/orgs/:id/activities
pub async fn list_activities<OR, MR, UR, TS, ID, AR>(
    State(org_service): State<Arc<OrgService<OR, MR, UR, TS, ID, AR>>>,
    Extension(claims): Extension<AuthClaims>,
    Path(org_id): Path<String>,
    Query(query): Query<ListActivitiesQuery>,
) -> Result<Json<Vec<ActivityResponseDto>>, (StatusCode, Json<ErrorResponse>)>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    org_service
        .list_activities(&org_id, &claims.user_id, query.limit, query.offset)
        .await
        .map(|activities| Json(activities.into_iter().map(|a| a.into()).collect()))
        .map_err(to_error_response)
}
