use serde::{Deserialize, Serialize};

use crate::modules::logging::domain::LogDomainError;

/// Filter preset ID wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterPresetId(String);

impl FilterPresetId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FilterPresetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Filter preset name (1-100 chars)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterPresetName(String);

impl FilterPresetName {
    pub fn new(name: String) -> Result<Self, LogDomainError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(LogDomainError::InvalidFilterPreset(
                "Name cannot be empty".to_string(),
            ));
        }
        if trimmed.len() > 100 {
            return Err(LogDomainError::InvalidFilterPreset(
                "Name cannot exceed 100 characters".to_string(),
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FilterPresetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Operators for metadata field queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetadataOperator {
    /// Equals (exact match)
    Eq,
    /// Not equals
    Neq,
    /// Contains (substring match, case-insensitive)
    Contains,
    /// Key exists in metadata
    Exists,
    /// Greater than (for numeric values)
    Gt,
    /// Less than (for numeric values)
    Lt,
    /// Greater than or equal (for numeric values)
    Gte,
    /// Less than or equal (for numeric values)
    Lte,
}

impl MetadataOperator {
    pub fn from_str(s: &str) -> Result<Self, LogDomainError> {
        match s.to_lowercase().as_str() {
            "eq" | "equals" | "=" => Ok(Self::Eq),
            "neq" | "ne" | "!=" => Ok(Self::Neq),
            "contains" | "like" => Ok(Self::Contains),
            "exists" => Ok(Self::Exists),
            "gt" | ">" => Ok(Self::Gt),
            "lt" | "<" => Ok(Self::Lt),
            "gte" | ">=" => Ok(Self::Gte),
            "lte" | "<=" => Ok(Self::Lte),
            _ => Err(LogDomainError::InvalidFilterPreset(format!(
                "Unknown metadata operator: {}",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Neq => "neq",
            Self::Contains => "contains",
            Self::Exists => "exists",
            Self::Gt => "gt",
            Self::Lt => "lt",
            Self::Gte => "gte",
            Self::Lte => "lte",
        }
    }

    /// Whether this operator requires a value
    pub fn requires_value(&self) -> bool {
        !matches!(self, Self::Exists)
    }
}

/// A single metadata field filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataFilter {
    /// The metadata key to filter on (e.g., "user_id", "request.path")
    pub key: String,
    /// The comparison operator
    pub operator: MetadataOperator,
    /// The value to compare against (None for Exists operator)
    pub value: Option<serde_json::Value>,
}

impl MetadataFilter {
    pub fn new(
        key: String,
        operator: MetadataOperator,
        value: Option<serde_json::Value>,
    ) -> Result<Self, LogDomainError> {
        if key.trim().is_empty() {
            return Err(LogDomainError::InvalidFilterPreset(
                "Metadata key cannot be empty".to_string(),
            ));
        }

        if operator.requires_value() && value.is_none() {
            return Err(LogDomainError::InvalidFilterPreset(format!(
                "Operator '{}' requires a value",
                operator.as_str()
            )));
        }

        Ok(Self {
            key: key.trim().to_string(),
            operator,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_preset_name_validation() {
        assert!(FilterPresetName::new("".to_string()).is_err());
        assert!(FilterPresetName::new("   ".to_string()).is_err());
        assert!(FilterPresetName::new("a".repeat(101)).is_err());
        assert!(FilterPresetName::new("Valid Name".to_string()).is_ok());
    }

    #[test]
    fn test_metadata_operator_parsing() {
        assert_eq!(
            MetadataOperator::from_str("eq").unwrap(),
            MetadataOperator::Eq
        );
        assert_eq!(
            MetadataOperator::from_str("contains").unwrap(),
            MetadataOperator::Contains
        );
        assert!(MetadataOperator::from_str("invalid").is_err());
    }

    #[test]
    fn test_metadata_filter_requires_value() {
        // Exists doesn't require value
        assert!(MetadataFilter::new("key".to_string(), MetadataOperator::Exists, None).is_ok());

        // Eq requires value
        assert!(MetadataFilter::new("key".to_string(), MetadataOperator::Eq, None).is_err());
        assert!(MetadataFilter::new(
            "key".to_string(),
            MetadataOperator::Eq,
            Some(serde_json::json!("value"))
        )
        .is_ok());
    }
}
