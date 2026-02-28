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
        use crate::core::constants::{SIG_CONFIRM_PREFIX, SIG_EXIT};
        let msg = get_command_translation("system.input.confirm_exit", &[]);
        Ok(format!("{}{}{}", SIG_CONFIRM_PREFIX, SIG_EXIT, msg))
    }

    fn priority(&self) -> u8 {
        100
    }
}
