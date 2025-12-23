pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use domain::{
    MemberId, OrgDomainError, OrgId, OrgName, OrgRole, OrgSlug, Organization,
    OrganizationMember, OrganizationMemberRepository, OrganizationRepository,
};
