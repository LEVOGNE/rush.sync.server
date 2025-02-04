// src/commands/handler.rs
use crate::commands::exit::exit::ExitCommand;

#[derive(Debug)]
pub struct CommandResult {
    pub message: String,
    pub success: bool,
    pub should_exit: bool,
}

pub struct CommandHandler {
    exit_command: ExitCommand,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            exit_command: ExitCommand::new(),
        }
    }

    pub fn handle_input(&self, input: &str) -> CommandResult {
        let input = input.trim();

        if input.is_empty() {
            return CommandResult {
                message: String::new(),
                success: false,
                should_exit: false,
            };
        }

        // Prüfe auf Exit-Befehl
        if self.exit_command.matches(input) {
            match self.exit_command.execute() {
                Ok(msg) => CommandResult {
                    message: msg,
                    success: true,
                    should_exit: true, // Signal zum Beenden
                },
                Err(e) => CommandResult {
                    message: format!("Fehler beim Beenden: {}", e),
                    success: false,
                    should_exit: false,
                },
            }
        } else {
            CommandResult {
                message: format!("Befehl unbekannt: {}", input),
                success: false,
                should_exit: false,
            }
        }
    }
}
