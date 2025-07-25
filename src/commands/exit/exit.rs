use crate::core::prelude::*;
use crate::i18n::get_command_translation;

pub struct ExitCommand;

impl ExitCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<String> {
        // ✅ RICHTIG: get_command_translation - gibt ASCII-Marker zurück
        let msg = get_command_translation("system.input.confirm_exit", &[]);
        Ok(format!("__CONFIRM_EXIT__{}", msg))
    }

    pub fn matches(&self, command: &str) -> bool {
        matches!(command.trim().to_lowercase().as_str(), "exit" | "q")
    }
}

impl Default for ExitCommand {
    fn default() -> Self {
        Self::new()
    }
}
