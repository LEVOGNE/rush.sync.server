// src/commands/version/version.rs
use crate::core::constants::VERSION;
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct VersionCommand;

impl VersionCommand {
    pub fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "version" | "ver")
    }

    pub fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Ok(get_command_translation(
            "system.commands.version",
            &[VERSION],
        ))
    }

    crate::async_fallback!();
}
