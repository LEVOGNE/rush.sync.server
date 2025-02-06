// src/commands/handler.rs
use crate::commands::clear::ClearCommand;
use crate::commands::exit::exit::ExitCommand;
use crate::commands::lang::lang::LanguageCommand;
use crate::i18n;

#[derive(Debug)]
pub struct CommandResult {
    pub message: String,
    pub success: bool,
    pub should_exit: bool,
}

pub struct CommandHandler {
    exit_command: ExitCommand,
    language_command: LanguageCommand,
    clear_command: ClearCommand,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            exit_command: ExitCommand::new(),
            language_command: LanguageCommand::new(),
            clear_command: ClearCommand::new(), // NEU
        }
    }

    pub fn handle_input(&self, input: &str) -> CommandResult {
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if input.is_empty() {
            return CommandResult {
                message: String::new(),
                success: false,
                should_exit: false,
            };
        }

        if self.exit_command.matches(input) {
            match self.exit_command.execute() {
                Ok(msg) => CommandResult {
                    message: msg,
                    success: true,
                    should_exit: true,
                },
                Err(e) => CommandResult {
                    message: format!("Fehler beim Beenden: {}", e),
                    success: false,
                    should_exit: false,
                },
            }
        } else if self.language_command.matches(parts[0]) {
            match self.language_command.execute(&parts[1..]) {
                Ok(msg) => CommandResult {
                    message: msg,
                    success: true,
                    should_exit: false,
                },
                Err(e) => CommandResult {
                    message: e.to_string(),
                    success: false,
                    should_exit: false,
                },
            }
        } else if self.clear_command.matches(parts[0]) {
            // NEU: Clear-Command Block
            match self.clear_command.execute() {
                Ok(msg) => CommandResult {
                    message: msg,
                    success: true,
                    should_exit: false,
                },
                Err(e) => CommandResult {
                    message: format!("Fehler beim Leeren des Outputs: {}", e),
                    success: false,
                    should_exit: false,
                },
            }
        } else {
            CommandResult {
                message: i18n::get_translation("system.commands.unknown", &[input]),
                success: false,
                should_exit: false,
            }
        }
    }
}
