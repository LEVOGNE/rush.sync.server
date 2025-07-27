// =====================================================
// COMMAND IMPLEMENTATIONS - TRAIT ADOPTION
// =====================================================

// =====================================================
// FILE: commands/history/history.rs - TRAIT IMPL
// =====================================================

use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct HistoryCommand;

impl Command for HistoryCommand {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Manage command history"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().starts_with("history")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"-c" | &"--clear") => Ok("__CLEAR_HISTORY__".to_string()),
            Some(&"-h" | &"--help") => Ok(get_translation("system.commands.history.help", &[])),
            _ => Ok(get_translation("system.commands.history.unknown", &[])),
        }
    }

    fn priority(&self) -> u8 {
        60 // Höhere Priorität für häufig genutzte Commands
    }
}
