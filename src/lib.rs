// =====================================================
// FILE: src/lib.rs - CLEAN VERSION OHNE PERFORMANCE
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
pub mod setup;
pub mod ui;

// Essential re-exports
pub use commands::{Command, CommandHandler, CommandRegistry};
pub use core::config::Config;
pub use core::error::{AppError, Result};

pub fn create_default_registry() -> CommandRegistry {
    use commands::{
        clear::ClearCommand, exit::ExitCommand, history::HistoryCommand, lang::LanguageCommand,
        log_level::LogLevelCommand, restart::RestartCommand, test::TestCommand,
        theme::ThemeCommand, version::VersionCommand,
    };

    let mut registry = CommandRegistry::new();

    // Core Commands
    registry.register(VersionCommand);
    registry.register(ClearCommand);
    registry.register(ExitCommand);
    registry.register(RestartCommand);

    // Configuration Commands
    registry.register(LogLevelCommand);
    registry.register(LanguageCommand::new());
    registry.register(ThemeCommand::new());

    // Utility Commands
    registry.register(HistoryCommand);
    registry.register(TestCommand);

    registry.initialize();
    registry
}

// Main entry point
pub async fn run() -> Result<()> {
    let config = Config::load().await?;
    let mut screen = ui::screen::ScreenManager::new(&config).await?;
    screen.run().await
}

pub use ui::screen::ScreenManager;

// Convenience functions
pub async fn run_with_config(config: Config) -> Result<()> {
    let mut screen = ScreenManager::new(&config).await?;
    screen.run().await
}

pub fn create_handler() -> CommandHandler {
    CommandHandler::new()
}

pub async fn load_config() -> Result<Config> {
    Config::load().await
}
