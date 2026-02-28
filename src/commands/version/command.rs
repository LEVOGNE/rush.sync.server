use crate::commands::command::Command;
use crate::core::constants::VERSION;
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct VersionCommand;

impl Command for VersionCommand {
    fn name(&self) -> &'static str {
        "version"
    }

    fn description(&self) -> &'static str {
        "Show application version"
    }

    fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "version" | "ver")
    }

    fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Ok(get_command_translation(
            "system.commands.version",
            &[VERSION],
        ))
    }

    fn priority(&self) -> u8 {
        40
    }
}
