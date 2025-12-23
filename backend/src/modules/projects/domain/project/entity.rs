use chrono::{DateTime, Utc};

use super::value_objects::{MetricsRetentionDays, ProjectId, ProjectName, RetentionDays, TracesRetentionDays};
use crate::modules::organizations::domain::OrgId;
use crate::modules::projects::domain::errors::ProjectDomainError;

/// Project - aggregate root representing a monitored application
#[derive(Debug, Clone)]
pub struct Project {
    id: ProjectId,
    organization_id: OrgId,
    name: ProjectName,
    description: Option<String>,
    retention_days: RetentionDays,
    metrics_retention_days: MetricsRetentionDays,
    traces_retention_days: TracesRetentionDays,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl Project {
    /// Create a new project
    pub fn new(
        id: ProjectId,
        organization_id: OrgId,
        name: ProjectName,
        description: Option<String>,
        retention_days: RetentionDays,
        metrics_retention_days: MetricsRetentionDays,
        traces_retention_days: TracesRetentionDays,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            organization_id,
            name,
            description,
            retention_days,
            metrics_retention_days,
            traces_retention_days,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Reconstruct from persistence layer
    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct(
        id: ProjectId,
        organization_id: OrgId,
        name: ProjectName,
        description: Option<String>,
        retention_days: RetentionDays,
        metrics_retention_days: MetricsRetentionDays,
        traces_retention_days: TracesRetentionDays,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            organization_id,
            name,
            description,
            retention_days,
            metrics_retention_days,
            traces_retention_days,
            created_at,
            updated_at,
            deleted_at,
        }
    }

    // Getters
    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn organization_id(&self) -> &OrgId {
        &self.organization_id
    }

    pub fn name(&self) -> &ProjectName {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn retention_days(&self) -> RetentionDays {
        self.retention_days
    }

    pub fn metrics_retention_days(&self) -> MetricsRetentionDays {
        self.metrics_retention_days
    }

    pub fn traces_retention_days(&self) -> TracesRetentionDays {
        self.traces_retention_days
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
    /// Update project details
    pub fn update(
        &mut self,
        name: Option<ProjectName>,
        description: Option<Option<String>>,
        retention_days: Option<RetentionDays>,
        metrics_retention_days: Option<MetricsRetentionDays>,
        traces_retention_days: Option<TracesRetentionDays>,
    ) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = description;
        }
        if let Some(retention_days) = retention_days {
            self.retention_days = retention_days;
        }
        if let Some(metrics_retention_days) = metrics_retention_days {
            self.metrics_retention_days = metrics_retention_days;
        }
        if let Some(traces_retention_days) = traces_retention_days {
            self.traces_retention_days = traces_retention_days;
        }
        self.updated_at = Utc::now();
    }

    /// Soft delete the project
    pub fn soft_delete(&mut self) -> Result<(), ProjectDomainError> {
        if self.deleted_at.is_some() {
            return Err(ProjectDomainError::ProjectAlreadyDeleted);
        }

        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> Project {
        let id = ProjectId::new("proj-123".to_string());
        let org_id = OrgId::new("org-456".to_string());
        let name = ProjectName::new("Test Project".to_string()).unwrap();
        Project::new(
            id,
            org_id,
            name,
            None,
            RetentionDays::default(),
            MetricsRetentionDays::default(),
            TracesRetentionDays::default(),
        )
    }

    #[test]
    fn test_new_project() {
        let project = create_test_project();
        assert!(!project.is_deleted());
        assert_eq!(project.name().as_str(), "Test Project");
        assert_eq!(project.retention_days().value(), 30);
        assert_eq!(project.metrics_retention_days().value(), 90);
        assert_eq!(project.traces_retention_days().value(), 14);
        assert!(project.description().is_none());
    }

    #[test]
    fn test_new_project_with_description() {
        let id = ProjectId::new("proj-123".to_string());
        let org_id = OrgId::new("org-456".to_string());
        let name = ProjectName::new("Test Project".to_string()).unwrap();
        let project = Project::new(
            id,
            org_id,
            name,
            Some("A test project".to_string()),
            RetentionDays::new(90).unwrap(),
            MetricsRetentionDays::new(60).unwrap(),
            TracesRetentionDays::new(7).unwrap(),
        );

        assert_eq!(project.description(), Some("A test project"));
        assert_eq!(project.retention_days().value(), 90);
        assert_eq!(project.metrics_retention_days().value(), 60);
        assert_eq!(project.traces_retention_days().value(), 7);
    }

    #[test]
    fn test_soft_delete() {
        let mut project = create_test_project();
        assert!(project.soft_delete().is_ok());
        assert!(project.is_deleted());
    }

    #[test]
    fn test_cannot_delete_twice() {
        let mut project = create_test_project();
        project.soft_delete().unwrap();

        assert!(matches!(
            project.soft_delete(),
            Err(ProjectDomainError::ProjectAlreadyDeleted)
        ));
    }

    #[test]
    fn test_update() {
        let mut project = create_test_project();
        let old_updated_at = project.updated_at();

        let new_name = ProjectName::new("Updated Project".to_string()).unwrap();
        project.update(
            Some(new_name),
            Some(Some("New description".to_string())),
            Some(RetentionDays::new(60).unwrap()),
            Some(MetricsRetentionDays::new(120).unwrap()),
            Some(TracesRetentionDays::new(30).unwrap()),
        );

        assert_eq!(project.name().as_str(), "Updated Project");
        assert_eq!(project.description(), Some("New description"));
        assert_eq!(project.retention_days().value(), 60);
        assert_eq!(project.metrics_retention_days().value(), 120);
        assert_eq!(project.traces_retention_days().value(), 30);
        assert!(project.updated_at() >= old_updated_at);
    }
}
