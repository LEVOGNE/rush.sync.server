// =====================================================
// FILE: src/commands/performance/command.rs - TRAIT IMPL
// =====================================================

use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct PerformanceCommand;

impl Command for PerformanceCommand {
    fn name(&self) -> &'static str {
        "performance"
    }

    fn description(&self) -> &'static str {
        "Show performance statistics and current config values"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd == "perf" || cmd == "performance" || cmd == "stats" || cmd.starts_with("perf ")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"--help" | &"-h") => Ok(crate::i18n::get_command_translation(
                "system.commands.performance.help",
                &[],
            )),
            None => super::manager::PerformanceManager::get_status(),
            _ => Ok(crate::i18n::get_command_translation(
                "system.commands.performance.unknown",
                &[],
            )),
        }
    }

    fn priority(&self) -> u8 {
        25 // Standard Priorität für System-Commands
    }
}
