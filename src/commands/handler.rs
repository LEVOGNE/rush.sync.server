// commands/handler.rs - WIRKLICH ALLGEMEIN (nur Delegation!)

use crate::commands::clear::ClearCommand;
use crate::commands::exit::exit::ExitCommand;
use crate::commands::history::HistoryCommand;
use crate::commands::lang::LanguageCommand;
use crate::commands::version::VersionCommand;
use crate::i18n;

#[derive(Debug)]
pub struct CommandResult {
    pub message: String,
    pub success: bool,
    pub should_exit: bool,
}

/// ✅ ALLGEMEIN: Enum delegiert nur, enthält keine spezifische Logic
#[derive(Debug)]
pub enum CommandType {
    History(HistoryCommand),
    Exit(ExitCommand),
    Language(LanguageCommand),
    Clear(ClearCommand),
    Version(VersionCommand),
    // ✅ SPÄTER: Einfach neue Commands hinzufügen
    // Auth(AuthCommand),
    // File(FileCommand),
    // Network(NetworkCommand),
    // ... 100+ weitere Commands
}

impl CommandType {
    /// ✅ ALLGEMEIN: Erstelle alle verfügbaren Commands
    pub fn all() -> Vec<Self> {
        vec![
            Self::History(HistoryCommand),
            Self::Exit(ExitCommand),
            Self::Language(LanguageCommand),
            Self::Clear(ClearCommand),
            Self::Version(VersionCommand),
        ]
    }

    /// ✅ ALLGEMEIN: Delegiert matching (keine spezifische Logic!)
    pub fn matches(&self, command: &str) -> bool {
        match self {
            Self::History(cmd) => cmd.matches(command),
            Self::Exit(cmd) => cmd.matches(command),
            Self::Language(cmd) => cmd.matches(command),
            Self::Clear(cmd) => cmd.matches(command),
            Self::Version(cmd) => cmd.matches(command),
        }
    }

    /// ✅ ALLGEMEIN: Delegiert sync execution (keine spezifische Logic!)
    pub fn execute_sync(&self, args: &[&str]) -> crate::core::error::Result<String> {
        match self {
            Self::History(cmd) => cmd.execute_sync(args),
            Self::Exit(cmd) => cmd.execute_sync(args),
            Self::Language(cmd) => cmd.execute_sync(args),
            Self::Clear(cmd) => cmd.execute_sync(args),
            Self::Version(cmd) => cmd.execute_sync(args),
        }
    }

    /// ✅ ALLGEMEIN: Delegiert async execution (keine spezifische Logic!)
    pub async fn execute_async(&self, args: &[&str]) -> crate::core::error::Result<String> {
        match self {
            Self::History(cmd) => cmd.execute_async(args).await,
            Self::Exit(cmd) => cmd.execute_async(args).await,
            Self::Language(cmd) => cmd.execute_async(args).await,
            Self::Clear(cmd) => cmd.execute_async(args).await,
            Self::Version(cmd) => cmd.execute_async(args).await,
        }
    }

    /// ✅ ALLGEMEIN: Delegiert async support check (keine spezifische Logic!)
    pub fn supports_async(&self) -> bool {
        match self {
            Self::History(cmd) => cmd.supports_async(),
            Self::Exit(cmd) => cmd.supports_async(),
            Self::Language(cmd) => cmd.supports_async(),
            Self::Clear(cmd) => cmd.supports_async(),
            Self::Version(cmd) => cmd.supports_async(),
        }
    }
}

/// ✅ ALLGEMEIN: Handler orchestriert nur, keine Command-spezifische Logic!
pub struct CommandHandler {
    commands: Vec<CommandType>,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            commands: CommandType::all(),
        }
    }

    /// ✅ ALLGEMEIN: SYNCHRONE Version (nur Orchestrierung!)
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

        // ✅ ALLGEMEIN: Iteriere über alle Commands
        for command in &self.commands {
            if command.matches(parts[0]) {
                match command.execute_sync(&parts[1..]) {
                    Ok(msg) => {
                        let should_exit = self.should_exit_on_message(&msg);
                        return CommandResult {
                            message: msg,
                            success: true,
                            should_exit,
                        };
                    }
                    Err(e) => {
                        return CommandResult {
                            message: e.to_string(),
                            success: false,
                            should_exit: false,
                        };
                    }
                }
            }
        }

        // ✅ ALLGEMEIN: Unbekannter Command
        CommandResult {
            message: i18n::get_command_translation("system.commands.unknown", &[input]),
            success: false,
            should_exit: false,
        }
    }

    /// ✅ ALLGEMEIN: ASYNC Version (nur Orchestrierung!)
    pub async fn handle_input_async(&self, input: &str) -> CommandResult {
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if input.is_empty() {
            return CommandResult {
                message: String::new(),
                success: false,
                should_exit: false,
            };
        }

        // ✅ ALLGEMEIN: Iteriere über alle Commands
        for command in &self.commands {
            if command.matches(parts[0]) {
                // ✅ ALLGEMEIN: Smart async/sync delegation
                let result = if command.supports_async() {
                    command.execute_async(&parts[1..]).await
                } else {
                    command.execute_sync(&parts[1..])
                };

                match result {
                    Ok(msg) => {
                        let should_exit = self.should_exit_on_message(&msg);
                        return CommandResult {
                            message: msg,
                            success: true,
                            should_exit,
                        };
                    }
                    Err(e) => {
                        return CommandResult {
                            message: e.to_string(),
                            success: false,
                            should_exit: false,
                        };
                    }
                }
            }
        }

        // ✅ ALLGEMEIN: Unbekannter Command
        CommandResult {
            message: i18n::get_command_translation("system.commands.unknown", &[input]),
            success: false,
            should_exit: false,
        }
    }

    /// ✅ ALLGEMEIN: Helper (keine Command-spezifische Logic!)
    fn should_exit_on_message(&self, message: &str) -> bool {
        message.starts_with("__EXIT__") || message.starts_with("__CONFIRM_EXIT__")
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
