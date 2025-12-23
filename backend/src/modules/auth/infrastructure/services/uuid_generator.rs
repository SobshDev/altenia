use uuid::Uuid;

use crate::modules::auth::application::ports::IdGenerator;

/// UUID v4 implementation of IdGenerator
pub struct UuidGenerator;

impl Default for UuidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl UuidGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for UuidGenerator {
    fn generate(&self) -> String {
        Uuid::new_v4().to_string()
    }
}
