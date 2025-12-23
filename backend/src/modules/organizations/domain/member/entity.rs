use chrono::{DateTime, Utc};

use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::organization::{MemberId, OrgId, OrgRole};

/// OrganizationMember - represents a user's membership in an organization
#[derive(Debug, Clone)]
pub struct OrganizationMember {
    id: MemberId,
    organization_id: OrgId,
    user_id: UserId,
    role: OrgRole,
    last_accessed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl OrganizationMember {
    /// Create a new membership
    pub fn new(id: MemberId, organization_id: OrgId, user_id: UserId, role: OrgRole) -> Self {
        let now = Utc::now();
        Self {
            id,
            organization_id,
            user_id,
            role,
            last_accessed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persistence layer
    pub fn reconstruct(
        id: MemberId,
        organization_id: OrgId,
        user_id: UserId,
        role: OrgRole,
        last_accessed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            user_id,
            role,
            last_accessed_at,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> &MemberId {
        &self.id
    }

    pub fn organization_id(&self) -> &OrgId {
        &self.organization_id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn role(&self) -> &OrgRole {
        &self.role
    }

    pub fn last_accessed_at(&self) -> Option<DateTime<Utc>> {
        self.last_accessed_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Behavior
    /// Update member's role
    pub fn update_role(&mut self, role: OrgRole) {
        self.role = role;
        self.updated_at = Utc::now();
    }

    /// Update last accessed timestamp (called when switching to this org)
    pub fn touch_last_accessed(&mut self) {
        self.last_accessed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_member() -> OrganizationMember {
        OrganizationMember::new(
            MemberId::new("member-123".to_string()),
            OrgId::new("org-456".to_string()),
            UserId::new("user-789".to_string()),
            OrgRole::Member,
        )
    }

    #[test]
    fn test_new_member() {
        let member = create_test_member();
        assert_eq!(member.role(), &OrgRole::Member);
        assert!(member.last_accessed_at().is_none());
    }

    #[test]
    fn test_update_role() {
        let mut member = create_test_member();
        member.update_role(OrgRole::Admin);
        assert_eq!(member.role(), &OrgRole::Admin);
    }

    #[test]
    fn test_touch_last_accessed() {
        let mut member = create_test_member();
        assert!(member.last_accessed_at().is_none());

        member.touch_last_accessed();
        assert!(member.last_accessed_at().is_some());
    }
}
