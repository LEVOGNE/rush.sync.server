// src/commands/exit/exit.rs
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct ExitCommand;

impl ExitCommand {
    pub fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "exit" | "q")
    }

    pub fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        let msg = get_command_translation("system.input.confirm_exit", &[]);
        Ok(format!("__CONFIRM_EXIT__{}", msg))
    }

    crate::async_fallback!();
}
