use chrono::{DateTime, Utc};
use serde_json::Value;

use super::value_objects::{AlertChannelId, ChannelType};
use crate::modules::projects::domain::ProjectId;

/// Alert Channel - notification destination
#[derive(Debug, Clone)]
pub struct AlertChannel {
    id: AlertChannelId,
    project_id: ProjectId,
    name: String,
    channel_type: ChannelType,
    config: Value,
    is_enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl AlertChannel {
    pub fn new(
        id: AlertChannelId,
        project_id: ProjectId,
        name: String,
        channel_type: ChannelType,
        config: Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            project_id,
            name,
            channel_type,
            config,
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from database
    #[allow(clippy::too_many_arguments)]
    pub fn from_db(
        id: AlertChannelId,
        project_id: ProjectId,
        name: String,
        channel_type: ChannelType,
        config: Value,
        is_enabled: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            project_id,
            name,
            channel_type,
            config,
            is_enabled,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> &AlertChannelId {
        &self.id
    }

    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn channel_type(&self) -> &ChannelType {
        &self.channel_type
    }

    pub fn config(&self) -> &Value {
        &self.config
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Mutators
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn update_config(&mut self, config: Value) {
        self.config = config;
        self.updated_at = Utc::now();
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
        self.updated_at = Utc::now();
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
        self.updated_at = Utc::now();
    }
}
