use std::collections::HashMap;
use std::sync::Arc;

use rand::Rng;

use crate::modules::auth::application::ports::{IdGenerator, OrgContext, TokenService};
use crate::modules::auth::domain::{AuthDomainError, Email, UserRepository, UserId};
use crate::modules::organizations::application::dto::*;
use crate::modules::organizations::domain::{
    ActivityId, ActivityType, MemberId, OrgActivity, OrgActivityRepository, OrgDomainError,
    OrgId, OrgName, OrgRole, OrgSlug, Organization, OrganizationMember,
    OrganizationMemberRepository, OrganizationRepository,
};

/// Organization service - orchestrates all organization use cases
pub struct OrgService<OR, MR, UR, TS, ID, AR>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    org_repo: Arc<OR>,
    member_repo: Arc<MR>,
    user_repo: Arc<UR>,
    token_service: Arc<TS>,
    id_generator: Arc<ID>,
    activity_repo: Arc<AR>,
}

impl<OR, MR, UR, TS, ID, AR> OrgService<OR, MR, UR, TS, ID, AR>
where
    OR: OrganizationRepository,
    MR: OrganizationMemberRepository,
    UR: UserRepository,
    TS: TokenService,
    ID: IdGenerator,
    AR: OrgActivityRepository,
{
    pub fn new(
        org_repo: Arc<OR>,
        member_repo: Arc<MR>,
        user_repo: Arc<UR>,
        token_service: Arc<TS>,
        id_generator: Arc<ID>,
        activity_repo: Arc<AR>,
    ) -> Self {
        Self {
            org_repo,
            member_repo,
            user_repo,
            token_service,
            id_generator,
            activity_repo,
        }
    }

    /// Generate a random 4-character suffix for slugs
    fn generate_random_suffix(&self) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::rng();
        (0..4)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Create a new organization
    pub async fn create_org(&self, cmd: CreateOrgCommand) -> Result<OrgResponse, OrgDomainError> {
        // 1. Validate name
        let name = OrgName::new(cmd.name)?;

        // 2. Generate slug with random suffix
        let suffix = self.generate_random_suffix();
        let slug = OrgSlug::generate(&name, &suffix);

        // 3. Create organization
        let org_id = OrgId::new(self.id_generator.generate());
        let org = Organization::new(org_id.clone(), name.clone(), slug);

        // 4. Save organization
        self.org_repo.save(&org).await?;

        // 5. Create membership as owner
        let user_id = UserId::new(cmd.user_id);
        let member_id = MemberId::new(self.id_generator.generate());
        let mut member = OrganizationMember::new(
            member_id,
            org_id.clone(),
            user_id.clone(),
            OrgRole::Owner,
        );
        member.touch_last_accessed();
        self.member_repo.save(&member).await?;

        // 6. Log activity
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id,
            ActivityType::OrgCreated,
            user_id,
            None,
            None,
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(OrgResponse {
            id: org.id().as_str().to_string(),
            name: org.name().as_str().to_string(),
            slug: org.slug().as_str().to_string(),
            is_personal: org.is_personal(),
            role: OrgRole::Owner.as_str().to_string(),
            created_at: org.created_at(),
        })
    }

    /// Create a personal organization (called during user registration)
    pub async fn create_personal_org(
        &self,
        cmd: CreatePersonalOrgCommand,
    ) -> Result<(OrgId, OrgRole), OrgDomainError> {
        // 1. Extract name from email prefix
        let email_prefix = cmd.email.split('@').next().unwrap_or("user");
        let name = OrgName::new(email_prefix.to_string())?;

        // 2. Generate slug with random suffix
        let suffix = self.generate_random_suffix();
        let slug = OrgSlug::generate(&name, &suffix);

        // 3. Create personal organization
        let org_id = OrgId::new(self.id_generator.generate());
        let org = Organization::new_personal(org_id.clone(), name, slug);

        // 4. Save organization
        self.org_repo.save(&org).await?;

        // 5. Create membership as owner
        let user_id = UserId::new(cmd.user_id);
        let member_id = MemberId::new(self.id_generator.generate());
        let mut member = OrganizationMember::new(
            member_id,
            org_id.clone(),
            user_id,
            OrgRole::Owner,
        );
        member.touch_last_accessed();
        self.member_repo.save(&member).await?;

        Ok((org_id, OrgRole::Owner))
    }

    /// List all organizations a user belongs to
    pub async fn list_user_orgs(&self, user_id: &str) -> Result<Vec<OrgResponse>, OrgDomainError> {
        let user_id = UserId::new(user_id.to_string());
        let memberships = self.member_repo.find_all_by_user(&user_id).await?;

        let mut orgs = Vec::new();
        for membership in memberships {
            if let Some(org) = self.org_repo.find_by_id(membership.organization_id()).await? {
                if !org.is_deleted() {
                    orgs.push(OrgResponse {
                        id: org.id().as_str().to_string(),
                        name: org.name().as_str().to_string(),
                        slug: org.slug().as_str().to_string(),
                        is_personal: org.is_personal(),
                        role: membership.role().as_str().to_string(),
                        created_at: org.created_at(),
                    });
                }
            }
        }

        Ok(orgs)
    }

    /// Get organization details (verify membership)
    pub async fn get_org(
        &self,
        org_id: &str,
        requesting_user_id: &str,
    ) -> Result<OrgResponse, OrgDomainError> {
        let org_id = OrgId::new(org_id.to_string());
        let user_id = UserId::new(requesting_user_id.to_string());

        // 1. Get organization
        let org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        if org.is_deleted() {
            return Err(OrgDomainError::OrgNotFound);
        }

        // 2. Verify membership
        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        Ok(OrgResponse {
            id: org.id().as_str().to_string(),
            name: org.name().as_str().to_string(),
            slug: org.slug().as_str().to_string(),
            is_personal: org.is_personal(),
            role: membership.role().as_str().to_string(),
            created_at: org.created_at(),
        })
    }

    /// Update organization (admin+ only)
    pub async fn update_org(&self, cmd: UpdateOrgCommand) -> Result<OrgResponse, OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let user_id = UserId::new(cmd.requesting_user_id);

        // 1. Get organization
        let mut org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        if org.is_deleted() {
            return Err(OrgDomainError::OrgNotFound);
        }

        // 2. Verify permission
        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        if !membership.role().can_update_org() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Update if name provided
        if let Some(new_name) = cmd.name {
            let old_name = org.name().as_str().to_string();
            let name = OrgName::new(new_name)?;
            let suffix = self.generate_random_suffix();
            let slug = OrgSlug::generate(&name, &suffix);
            org.update_name(name, slug);
            self.org_repo.save(&org).await?;

            // Log activity
            let mut metadata = HashMap::new();
            metadata.insert("old_name".to_string(), old_name);
            metadata.insert("new_name".to_string(), org.name().as_str().to_string());
            let activity = OrgActivity::new(
                ActivityId::new(self.id_generator.generate()),
                org_id.clone(),
                ActivityType::OrgNameChanged,
                user_id,
                None,
                Some(metadata),
            );
            let _ = self.activity_repo.save(&activity).await;
        }

        Ok(OrgResponse {
            id: org.id().as_str().to_string(),
            name: org.name().as_str().to_string(),
            slug: org.slug().as_str().to_string(),
            is_personal: org.is_personal(),
            role: membership.role().as_str().to_string(),
            created_at: org.created_at(),
        })
    }

    /// Delete organization (owner only, not personal org)
    pub async fn delete_org(&self, cmd: DeleteOrgCommand) -> Result<(), OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let user_id = UserId::new(cmd.requesting_user_id);

        // 1. Get organization
        let mut org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        // 2. Verify permission
        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        if !membership.role().can_delete_org() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Soft delete (will check if personal)
        org.soft_delete()?;
        self.org_repo.save(&org).await?;

        Ok(())
    }

    /// Add a member to an organization (admin+ only)
    pub async fn add_member(&self, cmd: AddMemberCommand) -> Result<MemberResponse, OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let requesting_user_id = UserId::new(cmd.requesting_user_id);

        // 1. Verify organization exists
        let org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        if org.is_deleted() {
            return Err(OrgDomainError::OrgNotFound);
        }

        // 2. Verify requester permission
        let requester_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &requesting_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        let new_role = OrgRole::from_str(&cmd.role)?;

        // Admin can add members, only owner can add admins
        if !requester_membership.role().can_manage_members() {
            return Err(OrgDomainError::InsufficientPermissions);
        }
        if new_role == OrgRole::Admin && !requester_membership.role().can_manage_admins() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 3. Find user by email
        let email = Email::new(cmd.email.clone())
            .map_err(|_| OrgDomainError::UserNotFound)?;
        let user = self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        let target_user_id = user.id().clone();

        // 4. Check if already a member
        if self
            .member_repo
            .find_by_org_and_user(&org_id, &target_user_id)
            .await?
            .is_some()
        {
            return Err(OrgDomainError::AlreadyMember);
        }

        // 5. Create membership
        let member_id = MemberId::new(self.id_generator.generate());
        let member = OrganizationMember::new(member_id, org_id.clone(), target_user_id.clone(), new_role);
        self.member_repo.save(&member).await?;

        // 6. Log activity
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id,
            ActivityType::MemberAdded,
            requesting_user_id,
            Some(target_user_id.clone()),
            None,
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(MemberResponse {
            id: member.id().as_str().to_string(),
            user_id: target_user_id.as_str().to_string(),
            email: cmd.email,
            display_name: user.display_name().map(|d| d.as_str().to_string()),
            role: new_role.as_str().to_string(),
            joined_at: member.created_at(),
        })
    }

    /// List members of an organization
    pub async fn list_members(
        &self,
        org_id: &str,
        requesting_user_id: &str,
    ) -> Result<Vec<MemberResponse>, OrgDomainError> {
        let org_id = OrgId::new(org_id.to_string());
        let user_id = UserId::new(requesting_user_id.to_string());

        // 1. Verify organization exists
        let org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        if org.is_deleted() {
            return Err(OrgDomainError::OrgNotFound);
        }

        // 2. Verify requester is a member
        self.member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 3. Get all members
        let memberships = self.member_repo.find_all_by_org(&org_id).await?;

        let mut members = Vec::new();
        for membership in memberships {
            // Get user email
            if let Some(user) = self
                .user_repo
                .find_by_id(membership.user_id())
                .await
                .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            {
                members.push(MemberResponse {
                    id: membership.id().as_str().to_string(),
                    user_id: membership.user_id().as_str().to_string(),
                    email: user.email().as_str().to_string(),
                    display_name: user.display_name().map(|d| d.as_str().to_string()),
                    role: membership.role().as_str().to_string(),
                    joined_at: membership.created_at(),
                });
            }
        }

        Ok(members)
    }

    /// Update a member's role (owner only)
    pub async fn update_member_role(
        &self,
        cmd: UpdateMemberRoleCommand,
    ) -> Result<MemberResponse, OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let requesting_user_id = UserId::new(cmd.requesting_user_id);
        let target_user_id = UserId::new(cmd.target_user_id.clone());

        // 1. Verify requester is owner
        let requester_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &requesting_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        if !requester_membership.role().can_manage_admins() {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 2. Get target membership
        let mut target_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &target_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 3. Parse new role
        let new_role = OrgRole::from_str(&cmd.new_role)?;
        let old_role = *target_membership.role();

        // 4. If demoting from owner, check not last owner
        if target_membership.role() == &OrgRole::Owner && new_role != OrgRole::Owner {
            let owner_count = self.member_repo.count_owners_for_update(&org_id).await?;
            if owner_count <= 1 {
                return Err(OrgDomainError::CannotDemoteLastOwner);
            }
        }

        // 5. Update role
        target_membership.update_role(new_role);
        self.member_repo.save(&target_membership).await?;

        // 6. Log activity
        let mut metadata = HashMap::new();
        metadata.insert("old_role".to_string(), old_role.as_str().to_string());
        metadata.insert("new_role".to_string(), new_role.as_str().to_string());
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id,
            ActivityType::MemberRoleChanged,
            requesting_user_id,
            Some(target_user_id.clone()),
            Some(metadata),
        );
        let _ = self.activity_repo.save(&activity).await;

        // Get user email for response
        let user = self
            .user_repo
            .find_by_id(&target_user_id)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        Ok(MemberResponse {
            id: target_membership.id().as_str().to_string(),
            user_id: cmd.target_user_id,
            email: user.email().as_str().to_string(),
            display_name: user.display_name().map(|d| d.as_str().to_string()),
            role: new_role.as_str().to_string(),
            joined_at: target_membership.created_at(),
        })
    }

    /// Remove a member from an organization
    pub async fn remove_member(&self, cmd: RemoveMemberCommand) -> Result<(), OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let requesting_user_id = UserId::new(cmd.requesting_user_id);
        let target_user_id = UserId::new(cmd.target_user_id);

        // 1. Verify requester membership
        let requester_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &requesting_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 2. Get target membership
        let target_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &target_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 3. Check permissions
        // - Owner can remove anyone
        // - Admin can remove members only
        match (requester_membership.role(), target_membership.role()) {
            (OrgRole::Owner, _) => {}
            (OrgRole::Admin, OrgRole::Member) => {}
            _ => return Err(OrgDomainError::InsufficientPermissions),
        }

        // 4. If removing owner, check not last owner
        if target_membership.role() == &OrgRole::Owner {
            let owner_count = self.member_repo.count_owners_for_update(&org_id).await?;
            if owner_count <= 1 {
                return Err(OrgDomainError::CannotRemoveLastOwner);
            }
        }

        // 5. Delete membership
        self.member_repo.delete(target_membership.id()).await?;

        // 6. Log activity
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id,
            ActivityType::MemberRemoved,
            requesting_user_id,
            Some(target_user_id),
            None,
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(())
    }

    /// Leave an organization
    pub async fn leave_org(&self, cmd: LeaveOrgCommand) -> Result<(), OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let user_id = UserId::new(cmd.user_id);

        // 1. Get membership
        let membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 2. If owner, check not last owner
        if membership.role() == &OrgRole::Owner {
            let owner_count = self.member_repo.count_owners_for_update(&org_id).await?;
            if owner_count <= 1 {
                return Err(OrgDomainError::CannotLeaveAsLastOwner);
            }
        }

        // 3. Delete membership
        self.member_repo.delete(membership.id()).await?;

        // 4. Log activity (user leaving is both actor and target)
        let mut metadata = HashMap::new();
        metadata.insert("reason".to_string(), "left".to_string());
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id,
            ActivityType::MemberRemoved,
            user_id.clone(),
            Some(user_id),
            Some(metadata),
        );
        let _ = self.activity_repo.save(&activity).await;

        Ok(())
    }

    /// Transfer ownership to another member
    pub async fn transfer_ownership(
        &self,
        cmd: TransferOwnershipCommand,
    ) -> Result<SwitchOrgResponse, OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id);
        let requesting_user_id = UserId::new(cmd.requesting_user_id);
        let new_owner_user_id = UserId::new(cmd.new_owner_user_id);

        // 1. Verify requester is owner
        let mut requester_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &requesting_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        if requester_membership.role() != &OrgRole::Owner {
            return Err(OrgDomainError::InsufficientPermissions);
        }

        // 2. Get new owner membership
        let mut new_owner_membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &new_owner_user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        let old_new_owner_role = *new_owner_membership.role();

        // 3. Promote new owner
        new_owner_membership.update_role(OrgRole::Owner);
        self.member_repo.save(&new_owner_membership).await?;

        // 4. Demote current owner to admin
        requester_membership.update_role(OrgRole::Admin);
        self.member_repo.save(&requester_membership).await?;

        // 5. Log activity for new owner promotion
        let mut metadata = HashMap::new();
        metadata.insert("old_role".to_string(), old_new_owner_role.as_str().to_string());
        metadata.insert("new_role".to_string(), OrgRole::Owner.as_str().to_string());
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id.clone(),
            ActivityType::MemberRoleChanged,
            requesting_user_id.clone(),
            Some(new_owner_user_id),
            Some(metadata),
        );
        let _ = self.activity_repo.save(&activity).await;

        // 6. Log activity for old owner demotion
        let mut metadata = HashMap::new();
        metadata.insert("old_role".to_string(), OrgRole::Owner.as_str().to_string());
        metadata.insert("new_role".to_string(), OrgRole::Admin.as_str().to_string());
        let activity = OrgActivity::new(
            ActivityId::new(self.id_generator.generate()),
            org_id.clone(),
            ActivityType::MemberRoleChanged,
            requesting_user_id.clone(),
            Some(requesting_user_id.clone()),
            Some(metadata),
        );
        let _ = self.activity_repo.save(&activity).await;

        // 7. Get organization details
        let org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        // 6. Get user email for token generation
        let user = self
            .user_repo
            .find_by_id(&requesting_user_id)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        // 7. Generate new tokens with updated role
        let org_context = Some(OrgContext {
            org_id: org.id().as_str().to_string(),
            org_role: requester_membership.role().as_str().to_string(),
        });

        let token_pair = self
            .token_service
            .generate_token_pair(&requesting_user_id, user.email().as_str(), org_context)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(SwitchOrgResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            expires_in: token_pair.access_expires_in,
            organization: OrgResponse {
                id: org.id().as_str().to_string(),
                name: org.name().as_str().to_string(),
                slug: org.slug().as_str().to_string(),
                is_personal: org.is_personal(),
                role: requester_membership.role().as_str().to_string(),
                created_at: org.created_at(),
            },
        })
    }

    /// Switch to a different organization
    pub async fn switch_org(
        &self,
        cmd: SwitchOrgCommand,
    ) -> Result<SwitchOrgResponse, OrgDomainError> {
        let org_id = OrgId::new(cmd.org_id.clone());
        let user_id = UserId::new(cmd.user_id.clone());

        // 1. Get organization
        let org = self
            .org_repo
            .find_by_id(&org_id)
            .await?
            .ok_or(OrgDomainError::OrgNotFound)?;

        if org.is_deleted() {
            return Err(OrgDomainError::OrgNotFound);
        }

        // 2. Get membership
        let mut membership = self
            .member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 3. Update last accessed
        membership.touch_last_accessed();
        self.member_repo.save(&membership).await?;

        // 4. Get user email for token
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?
            .ok_or(OrgDomainError::UserNotFound)?;

        // 5. Generate new tokens with org context
        let org_context = Some(OrgContext {
            org_id: org.id().as_str().to_string(),
            org_role: membership.role().as_str().to_string(),
        });

        let token_pair = self
            .token_service
            .generate_token_pair(&user_id, user.email().as_str(), org_context)
            .await
            .map_err(|e| OrgDomainError::InternalError(e.to_string()))?;

        Ok(SwitchOrgResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            expires_in: token_pair.access_expires_in,
            organization: OrgResponse {
                id: org.id().as_str().to_string(),
                name: org.name().as_str().to_string(),
                slug: org.slug().as_str().to_string(),
                is_personal: org.is_personal(),
                role: membership.role().as_str().to_string(),
                created_at: org.created_at(),
            },
        })
    }

    /// Get the default organization for a user (last accessed or personal)
    pub async fn get_default_org_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<(OrgId, OrgRole)>, OrgDomainError> {
        let user_id = UserId::new(user_id.to_string());

        // Try last accessed first
        if let Some(membership) = self.member_repo.find_last_accessed_by_user(&user_id).await? {
            // Verify org still exists and not deleted
            if let Some(org) = self.org_repo.find_by_id(membership.organization_id()).await? {
                if !org.is_deleted() {
                    return Ok(Some((
                        OrgId::new(org.id().as_str().to_string()),
                        *membership.role(),
                    )));
                }
            }
        }

        // Fall back to personal org
        if let Some(membership) = self.member_repo.find_personal_org_membership(&user_id).await? {
            return Ok(Some((
                OrgId::new(membership.organization_id().as_str().to_string()),
                *membership.role(),
            )));
        }

        Ok(None)
    }

    /// List activities for an organization
    pub async fn list_activities(
        &self,
        org_id: &str,
        requesting_user_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActivityResponse>, OrgDomainError> {
        let org_id = OrgId::new(org_id.to_string());
        let user_id = UserId::new(requesting_user_id.to_string());

        // 1. Verify membership
        self.member_repo
            .find_by_org_and_user(&org_id, &user_id)
            .await?
            .ok_or(OrgDomainError::NotOrgMember)?;

        // 2. Get activities
        let activities = self.activity_repo.find_by_org(&org_id, limit, offset).await?;

        // 3. Map to responses (look up emails for actor and target)
        let mut responses = Vec::new();
        for activity in activities {
            let actor = self
                .user_repo
                .find_by_id(activity.actor_id())
                .await
                .ok()
                .flatten();

            let target = if let Some(target_id) = activity.target_id() {
                self.user_repo.find_by_id(target_id).await.ok().flatten()
            } else {
                None
            };

            responses.push(ActivityResponse {
                id: activity.id().as_str().to_string(),
                activity_type: activity.activity_type().as_str().to_string(),
                actor_email: actor
                    .map(|u| {
                        u.display_name()
                            .map(|d| d.as_str().to_string())
                            .unwrap_or_else(|| u.email().as_str().to_string())
                    })
                    .unwrap_or_else(|| "Unknown".to_string()),
                target_email: target.map(|u| {
                    u.display_name()
                        .map(|d| d.as_str().to_string())
                        .unwrap_or_else(|| u.email().as_str().to_string())
                }),
                metadata: activity.metadata().cloned(),
                created_at: activity.created_at(),
            });
        }

        Ok(responses)
    }
}
