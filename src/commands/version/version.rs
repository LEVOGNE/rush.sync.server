use crate::core::constants::VERSION;
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct VersionCommand;

impl VersionCommand {
    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "version" | "ver")
    }

    pub fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Ok(get_command_translation(
            "system.commands.version",
            &[VERSION],
        ))
    }

    pub async fn execute_async(&self, args: &[&str]) -> Result<String> {
        // Default: nutze sync version
        self.execute_sync(args)
    }

    pub fn supports_async(&self) -> bool {
        false
    }
}
