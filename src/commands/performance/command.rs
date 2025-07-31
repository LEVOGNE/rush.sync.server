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
            Some(&"--help" | &"-h") => Ok("Performance Command Help:\n  perf                   Show performance status\n  performance           Same as perf\n  stats                 Same as perf\n  perf -h               Show this help".to_string()),
            None => super::manager::PerformanceManager::get_status(),
            _ => Ok("Unknown performance parameter. Use 'perf -h' for help.".to_string()),
        }
    }

    fn priority(&self) -> u8 {
        25
    }
}
