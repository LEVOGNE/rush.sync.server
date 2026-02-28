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
#[cfg(feature = "memory")]
pub mod embedded;
pub mod i18n;
pub mod input;
#[cfg(feature = "memory")]
pub mod memory;
pub mod output;
pub mod proxy;
pub mod server;
pub mod setup;
pub mod sync;
pub mod ui;

// Essential re-exports
pub use commands::{Command, CommandHandler, CommandRegistry};
pub use core::config::Config;
pub use core::error::{AppError, Result};
pub use ui::screen::ScreenManager;

pub fn create_default_registry() -> CommandRegistry {
    build_registry()
}

fn build_registry() -> CommandRegistry {
    use commands::{
        cleanup::CleanupCommand, clear::ClearCommand, create::CreateCommand, exit::ExitCommand,
        help::HelpCommand, history::HistoryCommand, lang::LanguageCommand, list::ListCommand,
        log_level::LogLevelCommand, recovery::RecoveryCommand, remote::RemoteCommand,
        restart::RestartCommand, start::StartCommand, stop::StopCommand, sync::SyncCommand,
        theme::ThemeCommand, version::VersionCommand,
    };

    let mut registry = CommandRegistry::new();

    registry
        .register(HelpCommand::new())
        .register(VersionCommand)
        .register(ClearCommand)
        .register(ExitCommand)
        .register(RestartCommand)
        .register(LogLevelCommand)
        .register(LanguageCommand::new())
        .register(ThemeCommand::new())
        .register(HistoryCommand)
        .register(RecoveryCommand::new())
        .register(RemoteCommand::new())
        .register(SyncCommand::new())
        .register(CleanupCommand::new())
        .register(CreateCommand::new())
        .register(ListCommand::new())
        .register(StartCommand::new())
        .register(StopCommand::new());

    #[cfg(feature = "memory")]
    registry.register(commands::memory::command::MemoryCommand::new());

    registry
}

pub async fn run() -> Result<()> {
    let config = Config::load().await?;
    run_with_config(config).await
}

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

#[cfg(test)]
pub fn create_test_registry() -> CommandRegistry {
    build_registry()
}
