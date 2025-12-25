use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::modules::auth::application::ports::IdGenerator;
use crate::modules::auth::domain::{Email, UserRepository, UserId};
use crate::modules::organizations::application::dto::*;
use crate::modules::organizations::domain::{
    ActivityId, ActivityType, InviteId, InviteStatus, MemberId, OrgActivity,
    OrgActivityRepository, OrgDomainError, OrgId, OrgRole, OrganizationInvite,
    OrganizationInviteRepository, OrganizationMember, OrganizationMemberRepository,
    OrganizationRepository,
};

/// Service for managing organization invites
pub struct InviteService<OR, MR, UR, IR, AR, ID>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    IR: OrganizationInviteRepository,
    AR: OrgActivityRepository,
    ID: IdGenerator,
{
    org_repo: Arc<OR>,
    member_repo: Arc<MR>,
    user_repo: Arc<UR>,
    invite_repo: Arc<IR>,
    activity_repo: Arc<AR>,
    id_generator: Arc<ID>,
}

impl<OR, MR, UR, IR, AR, ID> InviteService<OR, MR, UR, IR, AR, ID>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    IR: OrganizationInviteRepository,
    AR: OrgActivityRepository,
    ID: IdGenerator,
{
    pub fn new(
        org_repo: Arc<OR>,
        member_repo: Arc<MR>,
        user_repo: Arc<UR>,
        invite_repo: Arc<IR>,
        activity_repo: Arc<AR>,
        id_generator: Arc<ID>,
    ) -> Self {
        Self {
            org_repo,
            member_repo,
            user_repo,
            invite_repo,
            activity_repo,
            id_generator,
        }
    }

    /// Send an invite to join an organization
    pub async fn send_invite(&self, cmd: SendInviteCommand) -> Result<InviteResponse, OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id.clone());
        let inviter_user_id = UserId::new(cmd.inviter_user_id.clone());

        // 1. Verify org exists
        let org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        // 2. Verify requester is admin/owner
        let requester_member = self
            .member_repo
            .find_by_org_and_user(&org_id, &inviter_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        let requester_role = OrgRole::from_str(requester_member.role().as_str())?;
        if !requester_role.can_manage_members() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Validate role (can only invite admin or member, not owner)
        let invite_role = OrgRole::from_str(&cmd.role)?;
        if matches!(invite_role, OrgRole::Owner) {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 4. Check invitee email
        let invitee_email = Email::new(cmd.invitee_email.clone())
            .map_err(|_| OrgDomainError::InternalError("Invalid email".to_string()))?;

        // 5. Check if invitee is the inviter (cannot invite self)
        let inviter = self
            .user_repo
            .find_by_id(&inviter_user_id)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        if inviter.email().as_str() == invitee_email.as_str() {
            return Err(OrgDomainError::CannotInviteSelf);
        }

        // 6. Check if invitee exists and get their user_id
        let invitee = self
            .user_repo
            .find_by_email(&invitee_email)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        let invitee_id = invitee.as_ref().map(|u| u.id().as_str().to_string());

        // 7. If invitee exists, check their allow_invites setting
        if let Some(ref user) = invitee {
            if !user.allow_invites() {
                return Err(OrgDomainError::UserDoesNotAllowInvites);
            }

            // 8. Check if they're already a member
            let existing_member = self
                .member_repo
                .find_by_org_and_user(&org_id, user.id())
                .await?;
            if existing_member.is_some() {
                return Err(OrgDomainError::AlreadyMember);
            }
        }

        // 9. Check if pending invite already exists
        let existing_invite = self
            .invite_repo
            .find_pending_by_org_and_email(&cmd.org_id, invitee_email.as_str())
            .await?;
        if existing_invite.is_some() {
            return Err(OrgDomainError::InviteAlreadyExists);
        }

        // 10. Create invite (expires in 7 days)
        let invite_id = InviteId::new(self.id_generator.generate());
        let expires_at = Utc::now() + Duration::days(7);

        let invite = OrganizationInvite::new(
            invite_id,
            cmd.org_id.clone(),
            cmd.inviter_user_id.clone(),
            invitee_email.as_str().to_string(),
            invitee_id,
            invite_role,
            expires_at,
        );

        self.invite_repo.save(&invite).await?;

        // 11. Log activity
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org.id().clone(),
            ActivityType::InviteSent,
            inviter_user_id,
            None,
            Some(HashMap::from([
                ("invitee_email".to_string(), invitee_email.as_str().to_string()),
                ("role".to_string(), invite_role.as_str().to_string()),
            ])),
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(InviteResponse {
            id: invite.id().to_string(),
            organization_id: invite.organization_id().to_string(),
            organization_name: org.name().as_str().to_string(),
            inviter_email: inviter.email().as_str().to_string(),
            invitee_email: invite.invitee_email().to_string(),
            role: invite.role().as_str().to_string(),
            status: invite.status().as_str().to_string(),
            expires_at: invite.expires_at(),
            created_at: invite.created_at(),
        })
    }

    /// Accept an invite
    pub async fn accept_invite(&self, cmd: AcceptInviteCommand) -> Result<(), OrgDomainError> {
        // 1. Find invite
        let invite_id = InviteId::new(cmd.invite_id);
        let mut invite = self
            .invite_repo
            .find_by_id(&invite_id)
            .await?
            .ok_or(OrgDomainError::InviteNotFound)?;

        // 2. Verify invite is for this user
        let user = self
            .user_repo
            .find_by_id(&UserId::new(cmd.user_id.clone()))
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        if invite.invitee_id() != Some(user.id().as_str()) && invite.invitee_email() != user.email().as_str() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Check invite status
        if invite.status() != InviteStatus::Pending {
            return Err(OrgDomainError::InviteAlreadyProcessed);
        }

        // 4. Check if expired
        if invite.is_expired() {
            invite.mark_expired();
            self.invite_repo.update(&invite).await?;
            return Err(OrgDomainError::InviteExpired);
        }

        // 5. Check if already a member
        let invite_org_id = OrgId::new(invite.organization_id().to_string());
        let existing_member = self
            .member_repo
            .find_by_org_and_user(&invite_org_id, user.id())
            .await?;
        if existing_member.is_some() {
            // Already a member, just mark invite as accepted
            invite.accept();
            self.invite_repo.update(&invite).await?;
            return Err(OrgDomainError::AlreadyMember);
        }

        // 6. Create membership
        let member_id = MemberId::new(self.id_generator.generate());
        let member = OrganizationMember::new(
            member_id,
            invite.organization_id().to_string().into(),
            user.id().clone(),
            invite.role(),
        );
        self.member_repo.save(&member).await?;

        // 7. Mark invite as accepted
        invite.accept();
        self.invite_repo.update(&invite).await?;

        // 8. Log activity
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            invite.organization_id().to_string().into(),
            ActivityType::InviteAccepted,
            user.id().clone(),
            None,
            Some(HashMap::from([
                ("role".to_string(), invite.role().as_str().to_string()),
            ])),
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(())
    }

    /// Decline an invite
    pub async fn decline_invite(&self, cmd: DeclineInviteCommand) -> Result<(), OrgDomainError> {
        // 1. Find invite
        let invite_id = InviteId::new(cmd.invite_id);
        let mut invite = self
            .invite_repo
            .find_by_id(&invite_id)
            .await?
            .ok_or(OrgDomainError::InviteNotFound)?;

        // 2. Verify invite is for this user
        let user = self
            .user_repo
            .find_by_id(&UserId::new(cmd.user_id.clone()))
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        if invite.invitee_id() != Some(user.id().as_str()) && invite.invitee_email() != user.email().as_str() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Check invite status
        if invite.status() != InviteStatus::Pending {
            return Err(OrgDomainError::InviteAlreadyProcessed);
        }

        // 4. Mark as declined
        invite.decline();
        self.invite_repo.update(&invite).await?;

        // 5. Log activity
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            invite.organization_id().to_string().into(),
            ActivityType::InviteDeclined,
            user.id().clone(),
            None,
            None,
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(())
    }

    /// Cancel an invite (by org admin)
    pub async fn cancel_invite(&self, cmd: CancelInviteCommand) -> Result<(), OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id.clone());
        let requesting_user_id = UserId::new(cmd.requesting_user_id);

        // 1. Find invite
        let invite_id = InviteId::new(cmd.invite_id);
        let invite = self
            .invite_repo
            .find_by_id(&invite_id)
            .await?
            .ok_or(OrgDomainError::InviteNotFound)?;

        // 2. Verify invite belongs to this org
        if invite.organization_id() != cmd.org_id {
            return Err(OrgDomainError::InviteNotFound);
        }

        // 3. Verify requester is admin/owner
        let requester_member = self
            .member_repo
            .find_by_org_and_user(&org_id, &requesting_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        let requester_role = OrgRole::from_str(requester_member.role().as_str())?;
        if !requester_role.can_manage_members() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 4. Delete invite
        self.invite_repo.delete(&invite_id).await?;

        Ok(())
    }

    /// List pending invites for an organization
    pub async fn list_org_invites(
        &self,
        org_id: &str,
        requesting_user_id: &str,
    ) -> Result<Vec<InviteResponse>, OrgDomainError> {
        let org_id_typed = OrgId::new(org_id.to_string());
        let requesting_user_id_typed = UserId::new(requesting_user_id.to_string());

        // 1. Verify org exists
        let org = self
            .org_repo
            .find_by_id(&org_id_typed)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        // 2. Verify requester is admin/owner
        let requester_member = self
            .member_repo
            .find_by_org_and_user(&org_id_typed, &requesting_user_id_typed)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        let requester_role = OrgRole::from_str(requester_member.role().as_str())?;
        if !requester_role.can_manage_members() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Get pending invites
        let invites = self.invite_repo.list_pending_by_org(org_id).await?;

        // 4. Build responses (need to fetch inviter emails)
        let mut responses = Vec::new();
        for invite in invites {
            let inviter = self
                .user_repo
                .find_by_id(&UserId::new(invite.inviter_id().to_string()))
                .await
                .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

            let inviter_email = inviter
                .map(|u| u.email().as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            responses.push(InviteResponse {
                id: invite.id().to_string(),
                organization_id: invite.organization_id().to_string(),
                organization_name: org.name().as_str().to_string(),
                inviter_email,
                invitee_email: invite.invitee_email().to_string(),
                role: invite.role().as_str().to_string(),
                status: invite.status().as_str().to_string(),
                expires_at: invite.expires_at(),
                created_at: invite.created_at(),
            });
        }

        Ok(responses)
    }

    /// List pending invites for the current user
    pub async fn list_user_invites(&self, user_id: &str) -> Result<Vec<InviteResponse>, OrgDomainError> {
        let invites = self.invite_repo.list_pending_by_user(user_id).await?;

        let mut responses = Vec::new();
        for invite in invites {
            // Get org name
            let org = self
                .org_repo
                .find_by_id(&invite.organization_id().to_string().into())
                .await?;

            let org_name = org
                .map(|o| o.name().as_str().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            // Get inviter email
            let inviter = self
                .user_repo
                .find_by_id(&UserId::new(invite.inviter_id().to_string()))
                .await
                .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

            let inviter_email = inviter
                .map(|u| u.email().as_str().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            responses.push(InviteResponse {
                id: invite.id().to_string(),
                organization_id: invite.organization_id().to_string(),
                organization_name: org_name,
                inviter_email,
                invitee_email: invite.invitee_email().to_string(),
                role: invite.role().as_str().to_string(),
                status: invite.status().as_str().to_string(),
                expires_at: invite.expires_at(),
                created_at: invite.created_at(),
            });
        }

        Ok(responses)
    }

    /// Get count of pending invites for user (for badge)
    pub async fn count_user_invites(&self, user_id: &str) -> Result<InviteCountResponse, OrgDomainError> {
        let count = self.invite_repo.count_pending_by_user(user_id).await?;
        Ok(InviteCountResponse { count })
    }

    /// Get the invite repository reference (for cleanup task)
    pub fn invite_repo(&self) -> Arc<IR> {
        self.invite_repo.clone()
    }
}
