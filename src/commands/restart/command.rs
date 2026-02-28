use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::i18n::get_command_translation;

#[derive(Debug)]
pub struct RestartCommand;

impl Command for RestartCommand {
    fn name(&self) -> &'static str {
        "restart"
    }

    fn description(&self) -> &'static str {
        "Restart the application"
    }

    fn matches(&self, command: &str) -> bool {
        crate::matches_exact!(command, "restart" | "reboot" | "reset")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        use crate::core::constants::{SIG_CONFIRM_PREFIX, SIG_RESTART, SIG_RESTART_FORCE};
        match args.first() {
            Some(&"--help" | &"-h") => {
                Ok(get_command_translation("system.commands.restart.help", &[]))
            }
            Some(&"--force" | &"-f") => Ok(SIG_RESTART_FORCE.to_string()),
            None => {
                let msg = get_command_translation("system.commands.restart.confirm", &[]);
                Ok(format!("{}{}{}", SIG_CONFIRM_PREFIX, SIG_RESTART, msg))
            }
            _ => Ok(get_command_translation(
                "system.commands.restart.unknown",
                &[],
            )),
        }
    }

    fn priority(&self) -> u8 {
        90
    }
}
