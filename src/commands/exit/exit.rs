use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct ExitCommand;

impl ExitCommand {
    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "exit" | "q")
    }

    pub fn execute_sync(&self, _args: &[&str]) -> Result<String> {
        let msg = get_command_translation("system.input.confirm_exit", &[]);
        Ok(format!("__CONFIRM_EXIT__{}", msg))
    }

    pub async fn execute_async(&self, args: &[&str]) -> Result<String> {
        // Default: nutze sync version
        self.execute_sync(args)
    }

    pub fn supports_async(&self) -> bool {
        false
    }
}
