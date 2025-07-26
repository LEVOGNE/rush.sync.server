use crate::core::prelude::*;

pub struct HistoryCommand;

impl HistoryCommand {
    pub fn matches(&self, command: &str) -> bool {
        command.trim().starts_with("history")
    }

    pub fn execute(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"-c" | &"--clear") => Ok("__CLEAR_HISTORY__".to_string()),
            Some(&"-h" | &"--help") => Ok(get_translation("system.commands.history.help", &[])),
            _ => Ok(get_translation("system.commands.history.unknown", &[])),
        }
    }
}
