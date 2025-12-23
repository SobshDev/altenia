use crate::modules::projects::domain::errors::ProjectDomainError;

/// Project ID - wrapper around UUID string
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectId(String);

impl ProjectId {
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

impl From<String> for ProjectId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Project Name - validated name (1-100 chars, trimmed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName(String);

impl ProjectName {
    const MAX_LENGTH: usize = 100;

    pub fn new(name: String) -> Result<Self, ProjectDomainError> {
        let name = name.trim().to_string();

        if name.is_empty() {
            return Err(ProjectDomainError::InvalidProjectName(
                "name cannot be empty".to_string(),
            ));
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(ProjectDomainError::InvalidProjectName(format!(
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

/// Retention Days - validated retention period for logs (1-365 days)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionDays(i32);

impl RetentionDays {
    const MIN_DAYS: i32 = 1;
    const MAX_DAYS: i32 = 365;
    const DEFAULT_DAYS: i32 = 30;

    pub fn new(days: i32) -> Result<Self, ProjectDomainError> {
        if days < Self::MIN_DAYS || days > Self::MAX_DAYS {
            return Err(ProjectDomainError::InvalidRetentionDays(format!(
                "retention days must be between {} and {}",
                Self::MIN_DAYS,
                Self::MAX_DAYS
            )));
        }

        Ok(Self(days))
    }

    pub fn default_value() -> Self {
        Self(Self::DEFAULT_DAYS)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl Default for RetentionDays {
    fn default() -> Self {
        Self::default_value()
    }
}

/// Metrics Retention Days - validated retention period for metrics (1-365 days)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsRetentionDays(i32);

impl MetricsRetentionDays {
    const MIN_DAYS: i32 = 1;
    const MAX_DAYS: i32 = 365;
    const DEFAULT_DAYS: i32 = 90;

    pub fn new(days: i32) -> Result<Self, ProjectDomainError> {
        if days < Self::MIN_DAYS || days > Self::MAX_DAYS {
            return Err(ProjectDomainError::InvalidRetentionDays(format!(
                "metrics retention days must be between {} and {}",
                Self::MIN_DAYS,
                Self::MAX_DAYS
            )));
        }

        Ok(Self(days))
    }

    pub fn default_value() -> Self {
        Self(Self::DEFAULT_DAYS)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl Default for MetricsRetentionDays {
    fn default() -> Self {
        Self::default_value()
    }
}

/// Traces Retention Days - validated retention period for traces (1-90 days)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TracesRetentionDays(i32);

impl TracesRetentionDays {
    const MIN_DAYS: i32 = 1;
    const MAX_DAYS: i32 = 90;
    const DEFAULT_DAYS: i32 = 14;

    pub fn new(days: i32) -> Result<Self, ProjectDomainError> {
        if days < Self::MIN_DAYS || days > Self::MAX_DAYS {
            return Err(ProjectDomainError::InvalidRetentionDays(format!(
                "traces retention days must be between {} and {}",
                Self::MIN_DAYS,
                Self::MAX_DAYS
            )));
        }

        Ok(Self(days))
    }

    pub fn default_value() -> Self {
        Self(Self::DEFAULT_DAYS)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl Default for TracesRetentionDays {
    fn default() -> Self {
        Self::default_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_name() {
        assert!(ProjectName::new("My Project".to_string()).is_ok());
        assert!(ProjectName::new("  Trimmed  ".to_string()).is_ok());
        assert!(ProjectName::new("A".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_project_name() {
        assert!(ProjectName::new("".to_string()).is_err());
        assert!(ProjectName::new("   ".to_string()).is_err());
        assert!(ProjectName::new("A".repeat(101)).is_err());
    }

    #[test]
    fn test_valid_retention_days() {
        assert!(RetentionDays::new(1).is_ok());
        assert!(RetentionDays::new(30).is_ok());
        assert!(RetentionDays::new(365).is_ok());
    }

    #[test]
    fn test_invalid_retention_days() {
        assert!(RetentionDays::new(0).is_err());
        assert!(RetentionDays::new(-1).is_err());
        assert!(RetentionDays::new(366).is_err());
    }

    #[test]
    fn test_retention_days_default() {
        assert_eq!(RetentionDays::default().value(), 30);
    }
}
