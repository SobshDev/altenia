use crate::modules::organizations::domain::errors::OrgDomainError;

/// Activity ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActivityId(String);

impl ActivityId {
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

impl From<String> for ActivityId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Activity Type - the type of organization activity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityType {
    OrgCreated,
    MemberAdded,
    MemberRemoved,
    MemberRoleChanged,
    OrgNameChanged,
    InviteSent,
    InviteAccepted,
    InviteDeclined,
}

impl ActivityType {
    pub fn from_str(s: &str) -> Result<Self, OrgDomainError> {
        match s {
            "org_created" => Ok(Self::OrgCreated),
            "member_added" => Ok(Self::MemberAdded),
            "member_removed" => Ok(Self::MemberRemoved),
            "member_role_changed" => Ok(Self::MemberRoleChanged),
            "org_name_changed" => Ok(Self::OrgNameChanged),
            "invite_sent" => Ok(Self::InviteSent),
            "invite_accepted" => Ok(Self::InviteAccepted),
            "invite_declined" => Ok(Self::InviteDeclined),
            _ => Err(OrgDomainError::InvalidActivityType(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OrgCreated => "org_created",
            Self::MemberAdded => "member_added",
            Self::MemberRemoved => "member_removed",
            Self::MemberRoleChanged => "member_role_changed",
            Self::OrgNameChanged => "org_name_changed",
            Self::InviteSent => "invite_sent",
            Self::InviteAccepted => "invite_accepted",
            Self::InviteDeclined => "invite_declined",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_from_str() {
        assert_eq!(
            ActivityType::from_str("org_created").unwrap(),
            ActivityType::OrgCreated
        );
        assert_eq!(
            ActivityType::from_str("member_added").unwrap(),
            ActivityType::MemberAdded
        );
        assert_eq!(
            ActivityType::from_str("member_removed").unwrap(),
            ActivityType::MemberRemoved
        );
        assert_eq!(
            ActivityType::from_str("member_role_changed").unwrap(),
            ActivityType::MemberRoleChanged
        );
        assert_eq!(
            ActivityType::from_str("org_name_changed").unwrap(),
            ActivityType::OrgNameChanged
        );
        assert!(ActivityType::from_str("invalid").is_err());
    }

    #[test]
    fn test_activity_type_as_str() {
        assert_eq!(ActivityType::OrgCreated.as_str(), "org_created");
        assert_eq!(ActivityType::MemberAdded.as_str(), "member_added");
        assert_eq!(ActivityType::MemberRemoved.as_str(), "member_removed");
        assert_eq!(ActivityType::MemberRoleChanged.as_str(), "member_role_changed");
        assert_eq!(ActivityType::OrgNameChanged.as_str(), "org_name_changed");
    }
}
