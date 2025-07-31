use super::manager::LogLevelManager;
use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct LogLevelCommand;

impl Command for LogLevelCommand {
    fn name(&self) -> &'static str {
        "log-level"
    }

    fn description(&self) -> &'static str {
        "Change application log level"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd.starts_with("log-level")
            || cmd.starts_with("loglevel")
            || (cmd.starts_with("config") && cmd.contains("--log-level"))
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            None => Ok(LogLevelManager::show_status()),
            Some(&"--help" | &"-h" | &"help") => Ok(LogLevelManager::show_help()),
            Some(&level) => match LogLevelManager::set_level_persistent(level) {
                Ok(success_msg) => Ok(success_msg),
                Err(error_msg) => Ok(error_msg),
            },
        }
    }

    fn priority(&self) -> u8 {
        75
    }
}
