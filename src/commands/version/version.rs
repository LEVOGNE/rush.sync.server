use crate::core::constants::VERSION;
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

pub struct VersionCommand;

impl VersionCommand {
    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "version" | "ver")
    }

    pub fn execute(&self) -> Result<String> {
        Ok(get_command_translation(
            "system.commands.version",
            &[VERSION],
        ))
    }
}
