use std::fmt;
use std::str::FromStr;

use crate::modules::organizations::domain::OrgDomainError;

/// Unique identifier for an organization invite
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InviteId(String);

impl InviteId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InviteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<InviteId> for String {
    fn from(id: InviteId) -> Self {
        id.0
    }
}

/// Status of an organization invite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InviteStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
}

impl InviteStatus {
    pub fn as_str(&self) -> &str {
        match self {
            InviteStatus::Pending => "pending",
            InviteStatus::Accepted => "accepted",
            InviteStatus::Declined => "declined",
            InviteStatus::Expired => "expired",
        }
    }
}

impl FromStr for InviteStatus {
    type Err = OrgDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(InviteStatus::Pending),
            "accepted" => Ok(InviteStatus::Accepted),
            "declined" => Ok(InviteStatus::Declined),
            "expired" => Ok(InviteStatus::Expired),
            _ => Err(OrgDomainError::InvalidInviteStatus(s.to_string())),
        }
    }
}

impl fmt::Display for InviteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
