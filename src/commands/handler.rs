// src/commands/handler.rs
use crate::commands::clear::ClearCommand;
use crate::commands::exit::exit::ExitCommand;
use crate::commands::history::HistoryCommand;
use crate::commands::lang::lang::LanguageCommand;
use crate::commands::version::VersionCommand;
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
    version_command: VersionCommand,
    history_command: HistoryCommand,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            exit_command: ExitCommand::new(),
            language_command: LanguageCommand::new(),
            clear_command: ClearCommand::new(),
            version_command: VersionCommand::new(),
            history_command: HistoryCommand::new(),
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

        // History-Command hinzufügen
        if self.history_command.matches(parts[0]) {
            match self.history_command.execute(&parts[1..]) {
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
        } else if self.exit_command.matches(input) {
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
        } else if self.version_command.matches(parts[0]) {
            match self.version_command.execute() {
                Ok(msg) => CommandResult {
                    message: msg,
                    success: true,
                    should_exit: false,
                },
                Err(e) => CommandResult {
                    message: format!("Fehler beim Anzeigen der Version: {}", e),
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
