use crate::core::prelude::*;

#[derive(Debug)]
pub struct ClearCommand;

impl ClearCommand {
    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "clear" | "cls")
    }

    pub fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Ok("__CLEAR__".to_string())
    }

    pub async fn execute_async(&self, args: &[&str]) -> Result<String> {
        // Default: nutze sync version
        self.execute_sync(args)
    }

    pub fn supports_async(&self) -> bool {
        false
    }
}
