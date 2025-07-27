// =====================================================
// FILE: commands/exit/exit.rs - TRAIT IMPL
// =====================================================

use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct ExitCommand;

impl Command for ExitCommand {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn description(&self) -> &'static str {
        "Exit the application"
    }

    fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "exit" | "q")
    }

    fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        let msg = get_command_translation("system.input.confirm_exit", &[]);
        Ok(format!("__CONFIRM_EXIT__{}", msg))
    }

    fn priority(&self) -> u8 {
        100 // Höchste Priorität für Exit
    }
}
