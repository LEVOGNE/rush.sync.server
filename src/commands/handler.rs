// =====================================================
// FILE: src/commands/handler.rs - UNICODE-SAFE VERSION
// =====================================================

use super::registry::CommandRegistry;
use crate::i18n;

#[derive(Debug)]
pub struct CommandResult {
    pub message: String,
    pub success: bool,
    pub should_exit: bool,
}

pub struct CommandHandler {
    registry: CommandRegistry,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            registry: crate::create_default_registry(),
        }
    }

    pub fn with_registry(registry: CommandRegistry) -> Self {
        Self { registry }
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

        log::info!("🔧 CommandHandler processing: '{}'", input);

        match self.registry.execute_sync(parts[0], &parts[1..]) {
            Some(Ok(msg)) => {
                // ✅ UNICODE-SAFE PREVIEW - nutzt char boundaries
                let preview = if msg.chars().count() > 100 {
                    format!("{}...", msg.chars().take(97).collect::<String>())
                } else {
                    msg.clone()
                };

                log::info!(
                    "🔧 Command returned {} chars: '{}'",
                    msg.chars().count(), // ✅ char count statt byte count
                    preview
                );

                let should_exit = self.should_exit_on_message(&msg);
                let result = CommandResult {
                    message: msg,
                    success: true,
                    should_exit,
                };

                log::info!(
                    "🔧 CommandResult: success={}, message_len={}, should_exit={}",
                    result.success,
                    result.message.chars().count(), // ✅ char count statt byte count
                    result.should_exit
                );

                result
            }
            Some(Err(e)) => {
                log::error!("🔧 Command error: {}", e);
                CommandResult {
                    message: e.to_string(),
                    success: false,
                    should_exit: false,
                }
            }
            None => {
                log::warn!("🔧 Unknown command: {}", input);
                CommandResult {
                    message: crate::i18n::get_command_translation(
                        "system.commands.unknown",
                        &[input],
                    ),
                    success: false,
                    should_exit: false,
                }
            }
        }
    }

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

    pub fn add_command<T: crate::commands::command::Command>(&mut self, command: T) {
        self.registry.register(command);
    }

    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.registry.list_commands()
    }

    pub fn debug_info(&self) -> String {
        self.registry.debug_info()
    }

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
