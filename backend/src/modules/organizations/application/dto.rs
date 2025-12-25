use chrono::{DateTime, Utc};
use std::collections::HashMap;

// ==================== Commands ====================

/// Command to create a new organization
#[derive(Debug, Clone)]
pub struct CreateOrgCommand {
    pub name: String,
    pub user_id: String,
}

/// Command to create a personal organization (internal use during registration)
#[derive(Debug, Clone)]
pub struct CreatePersonalOrgCommand {
    pub user_id: String,
    pub email: String,
}

/// Command to update an organization
#[derive(Debug, Clone)]
pub struct UpdateOrgCommand {
    pub org_id: String,
    pub name: Option<String>,
    pub requesting_user_id: String,
}

/// Command to delete an organization
#[derive(Debug, Clone)]
pub struct DeleteOrgCommand {
    pub org_id: String,
    pub requesting_user_id: String,
}

/// Command to add a member to an organization
#[derive(Debug, Clone)]
pub struct AddMemberCommand {
    pub org_id: String,
    pub email: String,
    pub role: String,
    pub requesting_user_id: String,
}

/// Command to update a member's role
#[derive(Debug, Clone)]
pub struct UpdateMemberRoleCommand {
    pub org_id: String,
    pub target_user_id: String,
    pub new_role: String,
    pub requesting_user_id: String,
}

/// Command to remove a member from an organization
#[derive(Debug, Clone)]
pub struct RemoveMemberCommand {
    pub org_id: String,
    pub target_user_id: String,
    pub requesting_user_id: String,
}

/// Command for a user to leave an organization
#[derive(Debug, Clone)]
pub struct LeaveOrgCommand {
    pub org_id: String,
    pub user_id: String,
}

/// Command to transfer ownership to another member
#[derive(Debug, Clone)]
pub struct TransferOwnershipCommand {
    pub org_id: String,
    pub new_owner_user_id: String,
    pub requesting_user_id: String,
}

/// Command to switch to a different organization
#[derive(Debug, Clone)]
pub struct SwitchOrgCommand {
    pub org_id: String,
    pub user_id: String,
    pub device_fingerprint: String,
}

// ==================== Responses ====================

/// Response for organization data
#[derive(Debug, Clone)]
pub struct OrgResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

/// Response for member data
#[derive(Debug, Clone)]
pub struct MemberResponse {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

/// Response for organization switch
#[derive(Debug, Clone)]
pub struct SwitchOrgResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub organization: OrgResponse,
}

/// Response for activity data
#[derive(Debug, Clone)]
pub struct ActivityResponse {
    pub id: String,
    pub activity_type: String,
    pub actor_email: String,
    pub target_email: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
}

// ==================== Invite Commands ====================

/// Command to send an invite to join an organization
#[derive(Debug, Clone)]
pub struct SendInviteCommand {
    pub org_id: String,
    pub invitee_email: String,
    pub role: String,
    pub inviter_user_id: String,
}

/// Command to accept an invite
#[derive(Debug, Clone)]
pub struct AcceptInviteCommand {
    pub invite_id: String,
    pub user_id: String,
}

/// Command to decline an invite
#[derive(Debug, Clone)]
pub struct DeclineInviteCommand {
    pub invite_id: String,
    pub user_id: String,
}

/// Command to cancel an invite (by org admin)
#[derive(Debug, Clone)]
pub struct CancelInviteCommand {
    pub org_id: String,
    pub invite_id: String,
    pub requesting_user_id: String,
}

// ==================== Invite Responses ====================

/// Response for invite data
#[derive(Debug, Clone)]
pub struct InviteResponse {
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

/// Response for invite count
#[derive(Debug, Clone)]
pub struct InviteCountResponse {
    pub count: i64,
}
