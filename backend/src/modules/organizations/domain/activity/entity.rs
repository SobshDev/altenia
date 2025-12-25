use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::value_objects::{ActivityId, ActivityType};
use crate::modules::auth::domain::UserId;
use crate::modules::organizations::domain::organization::OrgId;

/// OrgActivity - represents an activity log entry for an organization
#[derive(Debug, Clone)]
pub struct OrgActivity {
    id: ActivityId,
    organization_id: OrgId,
    activity_type: ActivityType,
    actor_id: UserId,
    target_id: Option<UserId>,
    metadata: Option<HashMap<String, String>>,
    created_at: DateTime<Utc>,
}

impl OrgActivity {
    /// Create a new activity log entry
    pub fn new(
        id: ActivityId,
        organization_id: OrgId,
        activity_type: ActivityType,
        actor_id: UserId,
        target_id: Option<UserId>,
        metadata: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            id,
            organization_id,
            activity_type,
            actor_id,
            target_id,
            metadata,
            created_at: Utc::now(),
        }
    }

    /// Reconstruct from persistence layer
    pub fn reconstruct(
        id: ActivityId,
        organization_id: OrgId,
        activity_type: ActivityType,
        actor_id: UserId,
        target_id: Option<UserId>,
        metadata: Option<HashMap<String, String>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            organization_id,
            activity_type,
            actor_id,
            target_id,
            metadata,
            created_at,
        }
    }

    // Getters
    pub fn id(&self) -> &ActivityId {
        &self.id
    }

    pub fn organization_id(&self) -> &OrgId {
        &self.organization_id
    }

    pub fn activity_type(&self) -> ActivityType {
        self.activity_type
    }

    pub fn actor_id(&self) -> &UserId {
        &self.actor_id
    }

    pub fn target_id(&self) -> Option<&UserId> {
        self.target_id.as_ref()
    }

    pub fn metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_activity() -> OrgActivity {
        OrgActivity::new(
            ActivityId::new("activity-123".to_string()),
            OrgId::new("org-456".to_string()),
            ActivityType::MemberAdded,
            UserId::new("actor-789".to_string()),
            Some(UserId::new("target-012".to_string())),
            None,
        )
    }

    #[test]
    fn test_new_activity() {
        let activity = create_test_activity();
        assert_eq!(activity.id().as_str(), "activity-123");
        assert_eq!(activity.organization_id().as_str(), "org-456");
        assert_eq!(activity.activity_type(), ActivityType::MemberAdded);
        assert_eq!(activity.actor_id().as_str(), "actor-789");
        assert_eq!(activity.target_id().map(|t| t.as_str()), Some("target-012"));
        assert!(activity.metadata().is_none());
    }

    #[test]
    fn test_activity_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("old_role".to_string(), "member".to_string());
        metadata.insert("new_role".to_string(), "admin".to_string());

        let activity = OrgActivity::new(
            ActivityId::new("activity-123".to_string()),
            OrgId::new("org-456".to_string()),
            ActivityType::MemberRoleChanged,
            UserId::new("actor-789".to_string()),
            Some(UserId::new("target-012".to_string())),
            Some(metadata),
        );

        let meta = activity.metadata().unwrap();
        assert_eq!(meta.get("old_role"), Some(&"member".to_string()));
        assert_eq!(meta.get("new_role"), Some(&"admin".to_string()));
    }
}
