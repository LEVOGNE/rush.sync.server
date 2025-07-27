// =====================================================
// FILE: commands/handler.rs - BORROW-FEHLER BEHOBEN
// =====================================================

use super::registry::CommandRegistry;
use crate::i18n;

#[derive(Debug)]
pub struct CommandResult {
    pub message: String,
    pub success: bool,
    pub should_exit: bool,
}

/// ✅ ULTRA-SMART: Handler nutzt nur noch Registry!
pub struct CommandHandler {
    registry: CommandRegistry,
}

impl CommandHandler {
    /// ✅ EINFACHSTE INITIALISIERUNG
    pub fn new() -> Self {
        Self {
            registry: crate::create_default_registry(),
        }
    }

    /// ✅ CUSTOM Registry (für Tests/Extensions)
    pub fn with_registry(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    /// ✅ SYNCHRONE Verarbeitung - BORROW-SAFE
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

        // ✅ DELEGATION an Registry
        match self.registry.execute_sync(parts[0], &parts[1..]) {
            Some(Ok(msg)) => {
                let should_exit = self.should_exit_on_message(&msg);
                CommandResult {
                    message: msg,
                    success: true,
                    should_exit,
                }
            }
            Some(Err(e)) => CommandResult {
                message: e.to_string(),
                success: false,
                should_exit: false,
            },
            None => CommandResult {
                message: i18n::get_command_translation("system.commands.unknown", &[input]),
                success: false,
                should_exit: false,
            },
        }
    }

    /// ✅ ASYNC Verarbeitung - BORROW-SAFE
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

        // ✅ ASYNC DELEGATION an Registry
        match self.registry.execute_async(parts[0], &parts[1..]).await {
            Some(Ok(msg)) => {
                let should_exit = self.should_exit_on_message(&msg);
                CommandResult {
                    message: msg,
                    success: true,
                    should_exit,
                }
            }
            Some(Err(e)) => CommandResult {
                message: e.to_string(),
                success: false,
                should_exit: false,
            },
            None => CommandResult {
                message: i18n::get_command_translation("system.commands.unknown", &[input]),
                success: false,
                should_exit: false,
            },
        }
    }

    /// ✅ ERWEITERT: Command zu Registry hinzufügen (zur Laufzeit!)
    pub fn add_command<T: crate::commands::command::Command>(&mut self, command: T) {
        self.registry.register(command);
    }

    /// ✅ INFO: Registry Informationen
    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.registry.list_commands()
    }

    /// ✅ DEBUG: Registry Debug-Info
    pub fn debug_info(&self) -> String {
        self.registry.debug_info()
    }

    /// ✅ Helper bleibt gleich
    fn should_exit_on_message(&self, message: &str) -> bool {
        message.starts_with("__EXIT__")
            || message.starts_with("__CONFIRM_EXIT__")
            || message.starts_with("__RESTART__")
            || message.starts_with("__CONFIRM_RESTART__")
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}
