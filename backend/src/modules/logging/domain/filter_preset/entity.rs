use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::modules::auth::domain::UserId;
use crate::modules::logging::domain::log::{LogLevel, SortOrder};
use crate::modules::projects::domain::ProjectId;

use super::value_objects::{FilterPresetId, FilterPresetName, MetadataFilter};

/// Complete filter configuration stored in a preset
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Log levels to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<LogLevel>>,

    /// Start of time range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    /// End of time range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,

    /// Filter by source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Search text in message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    /// Filter by trace ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Metadata field filters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata_filters: Vec<MetadataFilter>,

    /// Sort order (default: descending/newest first)
    #[serde(default)]
    pub sort_order: SortOrder,
}

impl FilterConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_levels(mut self, levels: Vec<LogLevel>) -> Self {
        self.levels = Some(levels);
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_search(mut self, search: String) -> Self {
        self.search = Some(search);
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_metadata_filters(mut self, filters: Vec<MetadataFilter>) -> Self {
        self.metadata_filters = filters;
        self
    }

    pub fn with_sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    /// Check if this config has any active filters
    pub fn has_filters(&self) -> bool {
        self.levels.is_some()
            || self.start_time.is_some()
            || self.end_time.is_some()
            || self.source.is_some()
            || self.search.is_some()
            || self.trace_id.is_some()
            || !self.metadata_filters.is_empty()
    }
}

/// A saved filter preset
#[derive(Debug, Clone)]
pub struct FilterPreset {
    id: FilterPresetId,
    project_id: ProjectId,
    user_id: UserId,
    name: FilterPresetName,
    filter_config: FilterConfig,
    is_default: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl FilterPreset {
    /// Create a new filter preset
    pub fn new(
        id: FilterPresetId,
        project_id: ProjectId,
        user_id: UserId,
        name: FilterPresetName,
        filter_config: FilterConfig,
        is_default: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            project_id,
            user_id,
            name,
            filter_config,
            is_default,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persistence
    pub fn reconstruct(
        id: FilterPresetId,
        project_id: ProjectId,
        user_id: UserId,
        name: FilterPresetName,
        filter_config: FilterConfig,
        is_default: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            project_id,
            user_id,
            name,
            filter_config,
            is_default,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> &FilterPresetId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn name(&self) -> &FilterPresetName {
        &self.name
    }

    pub fn filter_config(&self) -> &FilterConfig {
        &self.filter_config
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Mutations
    pub fn update_name(&mut self, name: FilterPresetName) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn update_config(&mut self, config: FilterConfig) {
        self.filter_config = config;
        self.updated_at = Utc::now();
    }

    pub fn set_as_default(&mut self) {
        self.is_default = true;
        self.updated_at = Utc::now();
    }

    pub fn unset_default(&mut self) {
        self.is_default = false;
        self.updated_at = Utc::now();
    }
}
