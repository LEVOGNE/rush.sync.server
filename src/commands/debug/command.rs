// =====================================================
// FILE: src/commands/debug/command.rs - SICHERER DEBUG COMMAND
// =====================================================

use crate::commands::command::Command;
use crate::core::prelude::*;

#[derive(Debug)]
pub struct DebugCommand;

impl Command for DebugCommand {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn description(&self) -> &'static str {
        "Debug system status (safe - no loops)"
    }

    fn matches(&self, command: &str) -> bool {
        let cmd = command.trim().to_lowercase();
        cmd == "debug" || cmd.starts_with("debug ")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        match args.first() {
            Some(&"scroll") => {
                Ok("__DEBUG_SCROLL__📊 Scroll debug info → check STDERR".to_string())
            }
            Some(&"--help" | &"-h") => {
                Ok("Debug Commands:\n  debug scroll   Show scroll & viewport status (STDERR)\n  debug -h       Show this help".to_string())
            }
            None => {
                Ok("__DEBUG_SYSTEM__📊 System debug info → check STDERR".to_string())
            }
            _ => {
                Ok("Unknown debug command. Use 'debug -h' for help.".to_string())
            }
        }
    }

    fn priority(&self) -> u8 {
        15 // Mittlere Priorität für Debug-Commands
    }

    fn is_available(&self) -> bool {
        cfg!(debug_assertions) // Nur in Debug-builds
    }
}
