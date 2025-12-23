use chrono::{DateTime, Utc};

use super::value_objects::{OrgId, OrgName, OrgSlug};
use crate::modules::organizations::domain::errors::OrgDomainError;

/// Organization - aggregate root
#[derive(Debug, Clone)]
pub struct Organization {
    id: OrgId,
    name: OrgName,
    slug: OrgSlug,
    is_personal: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Organization {
    /// Create a new organization
    pub fn new(id: OrgId, name: OrgName, slug: OrgSlug) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            slug,
            is_personal: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Create a new personal organization
    pub fn new_personal(id: OrgId, name: OrgName, slug: OrgSlug) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            slug,
            is_personal: true,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Reconstruct from persistence layer
    pub fn reconstruct(
        id: OrgId,
        name: OrgName,
        slug: OrgSlug,
        is_personal: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            name,
            slug,
            is_personal,
            created_at,
            updated_at,
            deleted_at,
        }
    }

    // Getters
    pub fn id(&self) -> &OrgId {
        &self.id
    }

    pub fn name(&self) -> &OrgName {
        &self.name
    }

    pub fn slug(&self) -> &OrgSlug {
        &self.slug
    }

    pub fn is_personal(&self) -> bool {
        self.is_personal
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    // Behavior
    /// Update organization name and slug
    pub fn update_name(&mut self, name: OrgName, new_slug: OrgSlug) {
        self.name = name;
        self.slug = new_slug;
        self.updated_at = Utc::now();
    }

    /// Soft delete the organization
    pub fn soft_delete(&mut self) -> Result<(), OrgDomainError> {
        if self.is_personal {
            return Err(OrgDomainError::CannotDeletePersonalOrg);
        }

        if self.deleted_at.is_some() {
            return Err(OrgDomainError::OrgAlreadyDeleted);
        }

        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_org() -> Organization {
        let id = OrgId::new("org-123".to_string());
        let name = OrgName::new("Test Org".to_string()).unwrap();
        let slug = OrgSlug::generate(&name, "abcd");
        Organization::new(id, name, slug)
    }

    #[test]
    fn test_new_organization() {
        let org = create_test_org();
        assert!(!org.is_personal());
        assert!(!org.is_deleted());
        assert_eq!(org.name().as_str(), "Test Org");
    }

    #[test]
    fn test_new_personal_organization() {
        let id = OrgId::new("org-456".to_string());
        let name = OrgName::new("john".to_string()).unwrap();
        let slug = OrgSlug::generate(&name, "efgh");
        let org = Organization::new_personal(id, name, slug);

        assert!(org.is_personal());
        assert!(!org.is_deleted());
    }

    #[test]
    fn test_soft_delete() {
        let mut org = create_test_org();
        assert!(org.soft_delete().is_ok());
        assert!(org.is_deleted());
    }

    #[test]
    fn test_cannot_delete_personal_org() {
        let id = OrgId::new("org-456".to_string());
        let name = OrgName::new("john".to_string()).unwrap();
        let slug = OrgSlug::generate(&name, "efgh");
        let mut org = Organization::new_personal(id, name, slug);

        assert!(matches!(
            org.soft_delete(),
            Err(OrgDomainError::CannotDeletePersonalOrg)
        ));
    }

    #[test]
    fn test_cannot_delete_twice() {
        let mut org = create_test_org();
        org.soft_delete().unwrap();

        assert!(matches!(
            org.soft_delete(),
            Err(OrgDomainError::OrgAlreadyDeleted)
        ));
    }

    #[test]
    fn test_update_name() {
        let mut org = create_test_org();
        let old_updated_at = org.updated_at();

        let new_name = OrgName::new("Updated Org".to_string()).unwrap();
        let new_slug = OrgSlug::generate(&new_name, "wxyz");
        org.update_name(new_name, new_slug);

        assert_eq!(org.name().as_str(), "Updated Org");
        assert!(org.updated_at() >= old_updated_at);
    }
}
