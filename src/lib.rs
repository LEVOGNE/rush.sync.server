// =====================================================
// FILE: src/lib.rs - PERFORMANCE COMMAND HINZUGEFÜGT
// =====================================================

// ✅ ALTE Macros (behalten für Kompatibilität)
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

// ✅ NEUE Macros für Command System
#[macro_export]
macro_rules! register_command {
    ($registry:expr, $command:expr) => {
        $registry.register($command);
    };
}

#[macro_export]
macro_rules! register_commands {
    ($registry:expr, $($command:expr),+ $(,)?) => {
        $(
            $crate::register_command!($registry, $command);
        )+
    };
}

/// ✅ HAUPTMACRO - Erstellt vollständige Registry mit allen Standard-Commands
#[macro_export]
macro_rules! create_full_registry {
    () => {{
        use $crate::commands::{
            clear::ClearCommand,
            exit::exit::ExitCommand,
            history::HistoryCommand,
            lang::LanguageCommand,
            restart::RestartCommand,
            theme::ThemeCommand, // ✅ NEU HINZUGEFÜGT
            version::VersionCommand,
        };

        let mut registry = $crate::commands::registry::CommandRegistry::new();

        $crate::register_commands!(
            registry,
            HistoryCommand,
            ExitCommand,
            LanguageCommand,
            ClearCommand,
            RestartCommand,
            VersionCommand,
            ThemeCommand // ✅ NEU HINZUGEFÜGT
        );

        registry.initialize();
        registry
    }};
}

/// ✅ ERWEITERT - Registry mit Plugins
#[macro_export]
macro_rules! create_registry_with_plugins {
    ($($plugin:expr),+ $(,)?) => {{
        let mut registry = create_full_registry!();
        let mut plugin_manager = $crate::commands::PluginManager::new();

        $(
            plugin_manager.load_plugin($plugin);
        )+

        plugin_manager.apply_to_registry(&mut registry);
        (registry, plugin_manager)
    }};
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
pub use commands::{Command, CommandHandler, CommandPlugin, CommandRegistry, PluginManager};
pub use core::config::Config;
pub use core::error::{AppError, Result};

pub fn create_default_registry() -> CommandRegistry {
    use commands::{
        clear::ClearCommand, exit::exit::ExitCommand, history::HistoryCommand,
        lang::LanguageCommand, log_level::LogLevelCommand, performance::PerformanceCommand,
        restart::RestartCommand, theme::ThemeCommand, version::VersionCommand,
    };

    let mut registry = CommandRegistry::new();

    registry.register(HistoryCommand);
    registry.register(ExitCommand);
    registry.register(LogLevelCommand);
    registry.register(LanguageCommand::new()); // ✅ GEÄNDERT: ::new() hinzugefügt
    registry.register(ClearCommand);
    registry.register(RestartCommand);
    registry.register(VersionCommand);
    registry.register(PerformanceCommand);
    registry.register(ThemeCommand::new()); // ✅ GEÄNDERT: ::new() hinzugefügt

    registry.initialize();
    registry
}

// ✅ MAIN ENTRY POINT - für external usage
pub async fn run() -> Result<()> {
    let config = Config::load().await?;
    let mut screen = ui::screen::ScreenManager::new(&config).await?;
    screen.run().await
}

/// ✅ PUBLIC API
pub use ui::screen::ScreenManager;

// ✅ CONVENIENCE FUNCTIONS
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
