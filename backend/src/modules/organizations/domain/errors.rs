use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OrgDomainError {
    // Validation errors
    InvalidOrgName(String),
    InvalidOrgSlug(String),
    InvalidRole(String),
    InvalidActivityType(String),
    InvalidInviteStatus(String),

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

    // Invite errors
    InviteNotFound,
    InviteAlreadyExists,
    InviteExpired,
    InviteAlreadyProcessed,
    UserDoesNotAllowInvites,
    CannotInviteSelf,

    // Infrastructure errors
    InternalError(String),
}

impl fmt::Display for OrgDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOrgName(msg) => write!(f, "Invalid organization name: {}", msg),
            Self::InvalidOrgSlug(msg) => write!(f, "Invalid organization slug: {}", msg),
            Self::InvalidRole(msg) => write!(f, "Invalid role: {}", msg),
            Self::InvalidActivityType(msg) => write!(f, "Invalid activity type: {}", msg),
            Self::InvalidInviteStatus(msg) => write!(f, "Invalid invite status: {}", msg),
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
            Self::InviteNotFound => write!(f, "Invite not found"),
            Self::InviteAlreadyExists => write!(f, "An invite already exists for this user"),
            Self::InviteExpired => write!(f, "Invite has expired"),
            Self::InviteAlreadyProcessed => write!(f, "Invite has already been processed"),
            Self::UserDoesNotAllowInvites => write!(f, "User does not allow incoming invites"),
            Self::CannotInviteSelf => write!(f, "Cannot invite yourself"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for OrgDomainError {}
