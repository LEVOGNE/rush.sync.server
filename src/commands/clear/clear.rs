use crate::core::prelude::*;

pub struct ClearCommand;

impl ClearCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "clear" | "cls")
    }

    pub fn execute(&self) -> Result<String> {
        Ok("__CLEAR__".to_string())
    }
}

impl Default for ClearCommand {
    fn default() -> Self {
        Self::new()
    }
}
