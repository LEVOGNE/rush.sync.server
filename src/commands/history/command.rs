// =====================================================
// FILE: src/commands/history/command.rs - VEREINFACHT
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
            Some(&"-h" | &"--help") => Ok(format!(
                "📁 History Commands:\n\
                history        Show this help\n\
                history -c     Clear history\n\
                ↑ ↓           Navigate history\n\n\
                File: ~/.rss/rush.history"
            )),
            _ => Ok("📁 Use ↑↓ arrows to navigate, 'history -c' to clear".to_string()),
        }
    }

    fn priority(&self) -> u8 {
        60
    }
}
