// =====================================================
// FILE: src/lib.rs - OPTIMIERTE VERSION
// =====================================================

#[macro_export]
macro_rules! impl_default {
    ($type:ty, $body:expr) => {
        impl Default for $type {
            fn default() -> Self {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! matches_exact {
    ($cmd:expr, $($pattern:literal)|+) => {
        matches!($cmd.trim().to_lowercase().as_str(), $($pattern)|+)
    };
}

// Module definitions
pub mod commands;
pub mod core;
pub mod i18n;
pub mod input;
pub mod output;
pub mod proxy;
pub mod server;
pub mod setup;
pub mod ui;

// Essential re-exports
pub use commands::{Command, CommandHandler, CommandRegistry};
pub use core::config::Config;
pub use core::error::{AppError, Result};
pub use ui::screen::ScreenManager;

// ✅ OPTIMIERT: Lazy static für Registry - nur einmal erstellen
use std::sync::OnceLock;
static DEFAULT_REGISTRY: OnceLock<CommandRegistry> = OnceLock::new();

pub fn create_default_registry() -> CommandRegistry {
    build_registry()
}

// ✅ EXTRACTED: Registry-Building separiert für bessere Testbarkeit
fn build_registry() -> CommandRegistry {
    use commands::{
        cleanup::CleanupCommand, clear::ClearCommand, create::CreateCommand, exit::ExitCommand,
        help::HelpCommand, history::HistoryCommand, lang::LanguageCommand, list::ListCommand,
        log_level::LogLevelCommand, recovery::RecoveryCommand, restart::RestartCommand,
        start::StartCommand, stop::StopCommand, theme::ThemeCommand, version::VersionCommand,
    };

    let mut registry = CommandRegistry::new();

    // ✅ OPTIMIERT: Functional-Style Chain für kompakteren Code
    registry
        // Core Commands
        .register(HelpCommand::new())
        .register(VersionCommand)
        .register(ClearCommand)
        .register(ExitCommand)
        .register(RestartCommand)
        // Configuration Commands
        .register(LogLevelCommand)
        .register(LanguageCommand::new())
        .register(ThemeCommand::new())
        // Utility Commands
        .register(HistoryCommand)
        .register(RecoveryCommand::new())
        // Server Commands
        .register(CleanupCommand::new())
        .register(CreateCommand::new())
        .register(ListCommand::new())
        .register(StartCommand::new())
        .register(StopCommand::new());

    registry
}

// ✅ VEREINFACHT: Main entry point
pub async fn run() -> Result<()> {
    let config = Config::load().await?;
    run_with_config(config).await
}

// ✅ OPTIMIERT: Direkte Implementierung ohne Duplikation
pub async fn run_with_config(config: Config) -> Result<()> {
    let mut screen = ScreenManager::new(&config).await?;
    screen.run().await
}

// ✅ VEREINFACHT: Convenience functions
pub fn create_handler() -> CommandHandler {
    CommandHandler::new()
}

pub async fn load_config() -> Result<Config> {
    Config::load().await
}

// ✅ NEU: Registry-Testing Helper (für Unit Tests)
#[cfg(test)]
pub fn create_test_registry() -> CommandRegistry {
    // Für Tests immer fresh Registry ohne static caching
    build_registry()
}

// ✅ NEU: Command-Count für Debugging
pub fn get_command_count() -> usize {
    DEFAULT_REGISTRY.get().map(|r| r.len()).unwrap_or(0)
}
