use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn description(&self) -> &'static str {
        "Clear the screen"
    }

    fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "clear" | "cls")
    }

    fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        Ok(crate::core::constants::SIG_CLEAR.to_string())
    }

    fn priority(&self) -> u8 {
        80 // High priority for system command
    }
}
