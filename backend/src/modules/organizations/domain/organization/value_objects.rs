use crate::modules::organizations::domain::errors::OrgDomainError;

/// Organization ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrgId(String);

impl OrgId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for OrgId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Organization Name - validated name (1-100 chars, trimmed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgName(String);

impl OrgName {
    const MAX_LENGTH: usize = 100;

    pub fn new(name: String) -> Result<Self, OrgDomainError> {
        let name = name.trim().to_string();

        if name.is_empty() {
            return Err(OrgDomainError::InvalidOrgName(
                "name cannot be empty".to_string(),
            ));
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(OrgDomainError::InvalidOrgName(format!(
                "name cannot exceed {} characters",
                Self::MAX_LENGTH
            )));
        }

        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Organization Slug - URL-safe identifier (lowercase, alphanumeric + hyphens)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgSlug(String);

impl OrgSlug {
    /// Generate slug from organization name with a random suffix
    /// Example: "My Cool Org" + "x7k2" -> "my-cool-org-x7k2"
    pub fn generate(name: &OrgName, suffix: &str) -> Self {
        let base_slug: String = name
            .as_str()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();

        // Remove consecutive hyphens and trim hyphens from ends
        let base_slug = base_slug
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        let slug = format!("{}-{}", base_slug, suffix);
        Self(slug)
    }

    /// Create from existing slug (for reconstruction from DB)
    pub fn from_string(slug: String) -> Result<Self, OrgDomainError> {
        if slug.is_empty() {
            return Err(OrgDomainError::InvalidOrgSlug(
                "slug cannot be empty".to_string(),
            ));
        }

        // Validate slug format: lowercase alphanumeric and hyphens only
        if !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(OrgDomainError::InvalidOrgSlug(
                "slug must contain only lowercase letters, numbers, and hyphens".to_string(),
            ));
        }

        Ok(Self(slug))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Organization Role - defines permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrgRole {
    Owner,  // Full control, can delete org
    Admin,  // Can manage members, update org
    Member, // Read access, limited actions
}

impl OrgRole {
    pub fn from_str(s: &str) -> Result<Self, OrgDomainError> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            _ => Err(OrgDomainError::InvalidRole(format!(
                "unknown role: {}",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
        }
    }

    /// Can add/remove members (admin and owner)
    pub fn can_manage_members(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Can manage admins (owner only)
    pub fn can_manage_admins(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Can delete organization (owner only)
    pub fn can_delete_org(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Can update organization settings (admin and owner)
    pub fn can_update_org(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
}

/// Member ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberId(String);

impl MemberId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for MemberId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_org_name() {
        assert!(OrgName::new("My Organization".to_string()).is_ok());
        assert!(OrgName::new("  Trimmed  ".to_string()).is_ok());
        assert!(OrgName::new("A".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_org_name() {
        assert!(OrgName::new("".to_string()).is_err());
        assert!(OrgName::new("   ".to_string()).is_err());
        assert!(OrgName::new("A".repeat(101)).is_err());
    }

    #[test]
    fn test_slug_generation() {
        let name = OrgName::new("My Cool Org".to_string()).unwrap();
        let slug = OrgSlug::generate(&name, "x7k2");
        assert_eq!(slug.as_str(), "my-cool-org-x7k2");
    }

    #[test]
    fn test_slug_generation_with_special_chars() {
        let name = OrgName::new("Test & Company!".to_string()).unwrap();
        let slug = OrgSlug::generate(&name, "abc1");
        assert_eq!(slug.as_str(), "test-company-abc1");
    }

    #[test]
    fn test_org_role_from_str() {
        assert_eq!(OrgRole::from_str("owner").unwrap(), OrgRole::Owner);
        assert_eq!(OrgRole::from_str("ADMIN").unwrap(), OrgRole::Admin);
        assert_eq!(OrgRole::from_str("Member").unwrap(), OrgRole::Member);
        assert!(OrgRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_org_role_permissions() {
        assert!(OrgRole::Owner.can_manage_members());
        assert!(OrgRole::Owner.can_manage_admins());
        assert!(OrgRole::Owner.can_delete_org());

        assert!(OrgRole::Admin.can_manage_members());
        assert!(!OrgRole::Admin.can_manage_admins());
        assert!(!OrgRole::Admin.can_delete_org());

        assert!(!OrgRole::Member.can_manage_members());
        assert!(!OrgRole::Member.can_manage_admins());
        assert!(!OrgRole::Member.can_delete_org());
    }
}
