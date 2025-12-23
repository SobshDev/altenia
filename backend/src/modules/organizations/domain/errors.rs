use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OrgDomainError {
    // Validation errors
    InvalidOrgName(String),
    InvalidOrgSlug(String),
    InvalidRole(String),

    // Organization errors
    OrgNotFound,
    OrgAlreadyExists,
    SlugTaken,
    CannotDeletePersonalOrg,
    OrgAlreadyDeleted,

    // Membership errors
    NotOrgMember,
    AlreadyMember,
    UserNotFound,

    // Permission errors
    InsufficientPermissions,
    CannotRemoveLastOwner,
    CannotLeaveAsLastOwner,
    CannotDemoteLastOwner,

    // Infrastructure errors
    InternalError(String),
}

impl fmt::Display for OrgDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOrgName(msg) => write!(f, "Invalid organization name: {}", msg),
            Self::InvalidOrgSlug(msg) => write!(f, "Invalid organization slug: {}", msg),
            Self::InvalidRole(msg) => write!(f, "Invalid role: {}", msg),
            Self::OrgNotFound => write!(f, "Organization not found"),
            Self::OrgAlreadyExists => write!(f, "Organization already exists"),
            Self::SlugTaken => write!(f, "Organization slug is already taken"),
            Self::CannotDeletePersonalOrg => write!(f, "Cannot delete personal organization"),
            Self::OrgAlreadyDeleted => write!(f, "Organization is already deleted"),
            Self::NotOrgMember => write!(f, "User is not a member of this organization"),
            Self::AlreadyMember => write!(f, "User is already a member of this organization"),
            Self::UserNotFound => write!(f, "User not found"),
            Self::InsufficientPermissions => write!(f, "Insufficient permissions for this action"),
            Self::CannotRemoveLastOwner => write!(f, "Cannot remove the last owner of the organization"),
            Self::CannotLeaveAsLastOwner => write!(f, "Cannot leave as the last owner of the organization"),
            Self::CannotDemoteLastOwner => write!(f, "Cannot demote the last owner of the organization"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for OrgDomainError {}
