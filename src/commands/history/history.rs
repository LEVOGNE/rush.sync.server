use crate::core::prelude::*;

#[derive(Debug)]
pub struct HistoryCommand;

impl HistoryCommand {
    pub fn matches(&self, command: &str) -> bool {
        command.trim().starts_with("history")
    }

    pub fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"-c" | &"--clear") => Ok("__CLEAR_HISTORY__".to_string()),
            Some(&"-h" | &"--help") => Ok(get_translation("system.commands.history.help", &[])),
            _ => Ok(get_translation("system.commands.history.unknown", &[])),
        }
    }

    pub async fn execute_async(&self, args: &[&str]) -> Result<String> {
        // Default: nutze sync version
        self.execute_sync(args)
    }

    pub fn supports_async(&self) -> bool {
        false
    }
}
