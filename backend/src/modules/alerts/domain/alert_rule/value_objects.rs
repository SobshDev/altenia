use crate::modules::alerts::domain::AlertDomainError;

/// Alert Rule ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlertRuleId(String);

impl AlertRuleId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Rule Type - what kind of condition to evaluate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleType {
    /// Percentage of error/fatal logs
    ErrorRate,
    /// Total log count matching criteria
    LogCount,
    /// Logs matching a pattern
    PatternMatch,
}

impl RuleType {
    pub fn from_str(s: &str) -> Result<Self, AlertDomainError> {
        match s.to_lowercase().as_str() {
            "error_rate" => Ok(Self::ErrorRate),
            "log_count" => Ok(Self::LogCount),
            "pattern_match" => Ok(Self::PatternMatch),
            _ => Err(AlertDomainError::InvalidRuleType(format!(
                "Unknown rule type: {}. Valid types: error_rate, log_count, pattern_match",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ErrorRate => "error_rate",
            Self::LogCount => "log_count",
            Self::PatternMatch => "pattern_match",
        }
    }
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Threshold Operator - how to compare the value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThresholdOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl ThresholdOperator {
    pub fn from_str(s: &str) -> Result<Self, AlertDomainError> {
        match s.to_lowercase().as_str() {
            "gt" | ">" => Ok(Self::GreaterThan),
            "gte" | ">=" => Ok(Self::GreaterThanOrEqual),
            "lt" | "<" => Ok(Self::LessThan),
            "lte" | "<=" => Ok(Self::LessThanOrEqual),
            _ => Err(AlertDomainError::InvalidThresholdOperator(format!(
                "Unknown operator: {}. Valid operators: gt, gte, lt, lte",
                s
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GreaterThan => "gt",
            Self::GreaterThanOrEqual => "gte",
            Self::LessThan => "lt",
            Self::LessThanOrEqual => "lte",
        }
    }

    /// Evaluate the threshold condition
    pub fn evaluate(&self, actual: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan => actual > threshold,
            Self::GreaterThanOrEqual => actual >= threshold,
            Self::LessThan => actual < threshold,
            Self::LessThanOrEqual => actual <= threshold,
        }
    }
}

impl std::fmt::Display for ThresholdOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
