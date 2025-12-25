use chrono::{DateTime, Utc};

use super::value_objects::{InviteId, InviteStatus};
use crate::modules::organizations::domain::OrgRole;

/// Represents a pending organization membership invitation
#[derive(Debug, Clone)]
pub struct OrganizationInvite {
    id: InviteId,
    organization_id: String,
    inviter_id: String,
    invitee_email: String,
    invitee_id: Option<String>,
    role: OrgRole,
    status: InviteStatus,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OrganizationInvite {
    /// Create a new pending invite
    pub fn new(
        id: InviteId,
        organization_id: String,
        inviter_id: String,
        invitee_email: String,
        invitee_id: Option<String>,
        role: OrgRole,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            organization_id,
            inviter_id,
            invitee_email,
            invitee_id,
            role,
            status: InviteStatus::Pending,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persistence
    pub fn reconstruct(
        id: InviteId,
        organization_id: String,
        inviter_id: String,
        invitee_email: String,
        invitee_id: Option<String>,
        role: OrgRole,
        status: InviteStatus,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            inviter_id,
            invitee_email,
            invitee_id,
            role,
            status,
            expires_at,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> &InviteId {
        &self.id
    }

    pub fn organization_id(&self) -> &str {
        &self.organization_id
    }

    pub fn inviter_id(&self) -> &str {
        &self.inviter_id
    }

    pub fn invitee_email(&self) -> &str {
        &self.invitee_email
    }

    pub fn invitee_id(&self) -> Option<&str> {
        self.invitee_id.as_deref()
    }

    pub fn role(&self) -> OrgRole {
        self.role
    }

    pub fn status(&self) -> InviteStatus {
        self.status
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Check if invite has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Accept the invite
    pub fn accept(&mut self) {
        self.status = InviteStatus::Accepted;
        self.updated_at = Utc::now();
    }

    /// Decline the invite
    pub fn decline(&mut self) {
        self.status = InviteStatus::Declined;
        self.updated_at = Utc::now();
    }

    /// Mark as expired
    pub fn mark_expired(&mut self) {
        self.status = InviteStatus::Expired;
        self.updated_at = Utc::now();
    }
}
