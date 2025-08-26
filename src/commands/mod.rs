// =====================================================
// FILE: commands/mod.rs - CLEAN VERSION OHNE UNNÖTIGE PLUGINS
// =====================================================

pub mod cleanup;
pub mod clear;
pub mod command;
pub mod create;
pub mod exit;
pub mod handler;
pub mod history;
pub mod lang;
pub mod list;
pub mod log_level;
pub mod registry;
pub mod restart;
pub mod start;
pub mod stop;
pub mod theme;
pub mod version;

// ✅ CLEAN EXPORTS - Nur was wirklich gebraucht wird
pub use command::Command;
pub use handler::CommandHandler;
pub use registry::CommandRegistry;

// Commands alphabetisch sortiert
pub use cleanup::CleanupCommand;
pub use create::CreateCommand;
pub use list::ListCommand;
pub use start::StartCommand;
pub use stop::StopCommand;
