use super::registry::CommandRegistry;
use crate::core::prelude::*;
use crate::i18n;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub message: String,
    pub success: bool,
    pub should_exit: bool,
}

pub struct CommandHandler {
    registry: Arc<CommandRegistry>,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(crate::create_default_registry()),
        }
    }

    pub fn with_registry(registry: CommandRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }

    pub fn with_shared_registry(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }

    pub fn handle_input(&self, input: &str) -> CommandResult {
        self.process_input(input, false)
    }

    pub async fn handle_input_async(&self, input: &str) -> CommandResult {
        self.process_input_async(input).await
    }

    pub fn add_command<T: crate::commands::command::Command>(&mut self, command: T) -> Result<()> {
        if let Some(registry) = Arc::get_mut(&mut self.registry) {
            registry.register(command);
            Ok(())
        } else {
            // Registry is shared - must create a new one
            log::warn!("Registry is shared, creating new instance with added command");

            let mut new_registry = CommandRegistry::new();

            let existing_commands = self.registry.list_commands();
            for (name, _) in existing_commands {
                // TODO: Commands are not cloneable - architectural limitation
                log::warn!("Cannot copy existing command: {}", name);
            }

            new_registry.register(command);
            self.registry = Arc::new(new_registry);

            Err(AppError::Validation(
                "Registry was shared, created new instance".to_string(),
            ))
        }
    }

    pub fn list_commands(&self) -> Vec<(&str, &str)> {
        self.registry.list_commands()
    }

    pub fn debug_info(&self) -> String {
        self.registry.debug_info()
    }
}

impl CommandHandler {
    fn validate_and_parse<'a>(
        &self,
        input: &'a str,
    ) -> std::result::Result<ParsedInput<'a>, CommandResult> {
        let input = input.trim();
        if input.is_empty() {
            return Err(CommandResult::empty());
        }
        if input.len() > 1000 {
            log::warn!("Command input too long: {} chars", input.len());
            return Err(CommandResult::error(
                "Command input too long (max 1000 characters)",
            ));
        }
        Ok(InputParser::parse(input))
    }

    fn process_input(&self, input: &str, _is_async: bool) -> CommandResult {
        let parts = match self.validate_and_parse(input) {
            Ok(p) => p,
            Err(result) => return result,
        };
        log::debug!("Processing command: '{}'", parts.command);

        match self.registry.execute_sync(parts.command, &parts.args) {
            Some(result) => self.process_command_result(result),
            None => self.create_unknown_command_result(input.trim()),
        }
    }

    async fn process_input_async(&self, input: &str) -> CommandResult {
        let parts = match self.validate_and_parse(input) {
            Ok(p) => p,
            Err(result) => return result,
        };
        log::debug!("Processing async command: '{}'", parts.command);

        match self
            .registry
            .execute_async(parts.command, &parts.args)
            .await
        {
            Some(result) => self.process_command_result(result),
            None => self.create_unknown_command_result(input.trim()),
        }
    }

    fn process_command_result(&self, result: Result<String>) -> CommandResult {
        match result {
            Ok(msg) => {
                if log::log_enabled!(log::Level::Debug) {
                    self.log_command_success(&msg);
                }

                CommandResult {
                    message: msg.clone(),
                    success: true,
                    should_exit: ExitChecker::should_exit(&msg),
                }
            }
            Err(e) => {
                log::error!("Command execution failed: {}", e);
                CommandResult::error(&e.to_string())
            }
        }
    }

    fn create_unknown_command_result(&self, input: &str) -> CommandResult {
        log::warn!("Unknown command: '{}'", input);
        CommandResult::error(&UnknownCommandCache::get_message(input))
    }

    fn log_command_success(&self, msg: &str) {
        let char_count = msg.chars().count();

        log::debug!("Command returned {} chars", char_count);

        // Full output only for large outputs like mem info or JSON
        if char_count > 500
            && (msg.starts_with("MEMORY SNAPSHOT") || msg.trim_start().starts_with('{'))
        {
            log::debug!("FULL COMMAND OUTPUT:\n{}", msg);
        }
    }
}

struct InputParser;

impl InputParser {
    fn parse(input: &str) -> ParsedInput<'_> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            ParsedInput {
                command: "",
                args: Vec::new(),
            }
        } else {
            ParsedInput {
                command: parts[0],
                args: parts[1..].to_vec(),
            }
        }
    }
}

struct ParsedInput<'a> {
    command: &'a str,
    args: Vec<&'a str>,
}

// Static exit checker
struct ExitChecker;

impl ExitChecker {
    fn should_exit(message: &str) -> bool {
        use crate::core::constants::*;
        [SIG_EXIT, SIG_CONFIRM_EXIT, SIG_RESTART, SIG_CONFIRM_RESTART]
            .iter()
            .any(|&prefix| message.starts_with(prefix))
    }
}

use std::sync::OnceLock;

struct UnknownCommandCache;

impl UnknownCommandCache {
    fn get_message(input: &str) -> String {
        static TEMPLATE: OnceLock<String> = OnceLock::new();

        let template = TEMPLATE
            .get_or_init(|| i18n::get_command_translation("system.commands.unknown", &["%INPUT%"]));

        template.replace("%INPUT%", input)
    }
}

impl CommandResult {
    pub fn empty() -> Self {
        Self {
            message: String::new(),
            success: false,
            should_exit: false,
        }
    }

    pub fn success(message: String) -> Self {
        Self {
            message,
            success: true,
            should_exit: false,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            message: message.to_string(),
            success: false,
            should_exit: false,
        }
    }

    pub fn exit(message: String) -> Self {
        Self {
            message,
            success: true,
            should_exit: true,
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_error(&self) -> bool {
        !self.success
    }

    pub fn has_message(&self) -> bool {
        !self.message.is_empty()
    }
}

// Clone is cheap: only clones the Arc
impl Clone for CommandHandler {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl Default for CommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler {
    pub fn create_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub fn with_shared_handler(handler: Arc<Self>) -> Self {
        Self {
            registry: Arc::clone(&handler.registry),
        }
    }
}
